//! A `Sync`, append-only bump arena with stable addresses — the *owned* backing
//! for argument memory when we are the one producing it (the server side), and a
//! freestanding allocator in its own right.
//!
//! The engine hands the host argument data in pools it owns; on the host side a
//! pool is a *foreign* handle we only read and free. When **we** are the engine,
//! the dual is an arena we allocate into and own outright: copy bytes in, get a
//! stable pointer back, and free the whole region at once on drop. That is all
//! this is.
//!
//! Design constraints, and how they're met:
//!   * **Stable addresses** — a [`Pooled`](crate::pool::Pooled) holds a raw
//!     `*const` into here, so allocated bytes must never move. Memory lives in
//!     individually heap-allocated [`Chunk`]s that are never relocated and never
//!     reset (there is no `reset` — a region dies only when the whole `Arena`
//!     drops), so every pointer stays valid for the arena's life. No `Pin`
//!     needed.
//!   * **`&self` allocation** — the arena is shared (`Arc`) by every `Pooled`
//!     minted from it, so allocation goes through `&self`. The hot path is a
//!     single relaxed `fetch_add` on the current chunk's cursor; only growing the
//!     chunk list (when a chunk fills) takes a lock.
//!   * **`Sync`** — so `Arc<Arena>` is `Send + Sync` and the `Pooled` it backs
//!     keeps its thread-safety. All mutation is atomic or under the grow lock,
//!     and bytes are immutable once allocated.
//!   * **Aligned, typed allocation** — OUT payloads point at arrays of `#[repr(C)]`
//!     structs (`Ustring`/`ImageRecord`), not just bytes, so every allocation is
//!     rounded to [`MAX_ALIGN`] and chunk bases are `MAX_ALIGN`-aligned, keeping
//!     every cursor offset aligned by construction.

#![allow(dead_code)] // wired up incrementally by the pool / server layers.

use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::sync::Mutex;

use crate::pool::Anchor;

/// The maximum alignment the arena guarantees. Every allocation is rounded up to
/// this, and every chunk base is aligned to it, so cursor offsets are always
/// aligned and any `T` with `align_of::<T>() <= MAX_ALIGN` can be placed.
const MAX_ALIGN: usize = 16;

/// First-chunk capacity — deliberately small, since each registered pool often
/// backs just one short payload (a command's strings, an error message), so a
/// flat 4 KB floor would be mostly slack. Chunks grow geometrically from here up
/// to [`MAX_CHUNK`]; a single request larger than the next chunk size gets its
/// own exact chunk.
const FIRST_CHUNK: usize = 256;

/// The capacity chunk growth doubles toward.
const MAX_CHUNK: usize = 64 * 1024;

/// One heap-allocated, never-moved span of bytes with its own bump cursor.
struct Chunk {
    base: NonNull<u8>,
    cap: usize,
    cursor: AtomicUsize,
}

impl Chunk {
    /// Allocate a chunk of `cap` bytes (rounded up to [`MAX_ALIGN`], and at least
    /// `MAX_ALIGN`), with a [`MAX_ALIGN`]-aligned base. The caller chooses the size
    /// (first / geometric / dedicated); `boxed` does not floor it.
    fn boxed(cap: usize) -> Box<Chunk> {
        let cap = cap.max(MAX_ALIGN).next_multiple_of(MAX_ALIGN);
        let layout = Layout::from_size_align(cap, MAX_ALIGN).expect("arena chunk layout");
        // SAFETY: `cap` is non-zero (>= MAX_ALIGN), so the layout has non-zero size.
        let raw = unsafe { alloc(layout) };
        let base = NonNull::new(raw).unwrap_or_else(|| handle_alloc_error(layout));
        Box::new(Chunk {
            base,
            cap,
            cursor: AtomicUsize::new(0),
        })
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.cap, MAX_ALIGN).expect("arena chunk layout");
        // SAFETY: `base`/`cap` are exactly what `boxed` allocated with this layout,
        // and nothing reads the bytes after the arena (sole owner) drops.
        unsafe { dealloc(self.base.as_ptr(), layout) }
    }
}

/// An append-only bump arena. Allocate with [`alloc_bytes`](Arena::alloc_bytes) /
/// [`alloc_slice`](Arena::alloc_slice); the whole region frees on drop.
pub(crate) struct Arena {
    /// The chunk the hot path bumps into. Points into a `Box` owned by `chunks`,
    /// so it stays valid as that `Vec` grows (the box's pointee never moves).
    current: AtomicPtr<Chunk>,
    /// Owns every chunk; locked only to grow the list or on drop.
    // The `Box` is load-bearing, not redundant: `current` is an `AtomicPtr<Chunk>`
    // into the chunk *behind* the box, so the box keeps that pointee fixed as this
    // `Vec` reallocates on growth. Without it the chunks would move and dangle.
    #[allow(clippy::vec_box)]
    chunks: Mutex<Vec<Box<Chunk>>>,
    /// Total bytes handed out (rounded reservations), for argument accounting —
    /// `library_report_allocated_arguments`' per-pool byte total.
    allocated: AtomicUsize,
}

// SAFETY: every field mutation is atomic (`current`, each chunk `cursor`) or under
// `chunks`'s lock; allocated bytes are immutable, so concurrent reads of distinct
// regions don't race. The raw `NonNull`/`*mut` fields are what opt the type out of
// the auto traits; the synchronization above is what makes sharing sound.
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Anchor for Arena {}

impl Arena {
    /// A new arena with one chunk ready.
    pub(crate) fn new() -> Arena {
        let first = Chunk::boxed(FIRST_CHUNK);
        // Grab the stable pointer to the chunk *behind* the box before moving the
        // box into the vec (the vec relocates the box pointer, never its pointee).
        let current = AtomicPtr::new(&*first as *const Chunk as *mut Chunk);
        Arena {
            current,
            chunks: Mutex::new(vec![first]),
            allocated: AtomicUsize::new(0),
        }
    }

    /// Total bytes reserved over this arena's life — what a registered pool
    /// contributes to `library_report_allocated_arguments`' byte total.
    pub(crate) fn allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Reserve `size` aligned bytes and return a pointer to their start.
    fn bump(&self, size: usize) -> NonNull<u8> {
        // Round up so the cursor stays MAX_ALIGN-aligned; never reserve zero (so
        // distinct allocations get distinct pointers).
        let size = size.next_multiple_of(MAX_ALIGN).max(MAX_ALIGN);
        loop {
            // SAFETY: `current` points at a chunk owned by `self.chunks`, alive for
            // this borrow of `&self`.
            let chunk = unsafe { &*self.current.load(Ordering::Acquire) };
            let offset = chunk.cursor.fetch_add(size, Ordering::Relaxed);
            if offset + size <= chunk.cap {
                self.allocated.fetch_add(size, Ordering::Relaxed);
                // SAFETY: `offset + size <= cap`, so the span is within the chunk.
                return unsafe { NonNull::new_unchecked(chunk.base.as_ptr().add(offset)) };
            }
            // Didn't fit — append a fresh chunk (or notice someone already did) and
            // retry. A new chunk always has cap >= size, so this terminates.
            self.grow(size);
        }
    }

    /// Cold path: make `current` point at a chunk with room for `size`.
    fn grow(&self, size: usize) {
        let mut chunks = self.chunks.lock().expect("arena grow lock");
        // Re-check under the lock: another thread may have grown while we raced
        // here, in which case `current` already has room and we leave it be.
        // SAFETY: same as in `bump` — `current` is a live, owned chunk.
        let chunk = unsafe { &*self.current.load(Ordering::Acquire) };
        if chunk.cursor.load(Ordering::Relaxed) + size <= chunk.cap {
            return;
        }
        // Grow geometrically (double, capped at MAX_CHUNK), but always at least
        // `size` so an oversized single request gets its own exact chunk.
        let next = (chunk.cap * 2).min(MAX_CHUNK).max(size);
        let fresh = Chunk::boxed(next);
        let ptr = &*fresh as *const Chunk as *mut Chunk;
        chunks.push(fresh);
        self.current.store(ptr, Ordering::Release);
    }

    /// Copy `src` into the arena; the returned slice pointer is stable for the
    /// arena's life.
    pub(crate) fn alloc_bytes(&self, src: &[u8]) -> *const [u8] {
        let dst = self.bump(src.len());
        // SAFETY: `dst` owns `>= src.len()` fresh bytes (just reserved, unaliased);
        // src and dst don't overlap (dst is freshly bumped).
        unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), src.len());
            core::ptr::slice_from_raw_parts(dst.as_ptr().cast_const(), src.len())
        }
    }

    /// Copy a `T` slice in (e.g. a `Ustring` / `ImageRecord` array a payload points
    /// at). `T: Copy` so there's nothing to drop — the arena frees raw bytes.
    pub(crate) fn alloc_slice<T: Copy>(&self, src: &[T]) -> *const [T] {
        assert!(
            core::mem::align_of::<T>() <= MAX_ALIGN,
            "arena alloc_slice: align_of::<T>() exceeds MAX_ALIGN",
        );
        let bytes = core::mem::size_of_val(src);
        let dst = self.bump(bytes).as_ptr().cast::<T>();
        // SAFETY: `dst` is MAX_ALIGN-aligned (>= align_of::<T>()) with room for
        // `src.len()` `T`s, freshly reserved and unaliased.
        unsafe {
            core::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
            core::ptr::slice_from_raw_parts(dst.cast_const(), src.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    /// Read back a slice pointer the arena handed out.
    unsafe fn as_bytes<'a>(ptr: *const [u8]) -> &'a [u8] {
        unsafe { &*ptr }
    }

    #[test]
    fn alloc_bytes_roundtrips() {
        let arena = Arena::new();
        let p = arena.alloc_bytes(b"frogans");
        assert_eq!(unsafe { as_bytes(p) }, b"frogans");
    }

    #[test]
    fn empty_alloc_is_valid_and_distinct() {
        let arena = Arena::new();
        let a = arena.alloc_bytes(b"");
        let b = arena.alloc_bytes(b"");
        assert_eq!(unsafe { as_bytes(a) }.len(), 0);
        assert_eq!(unsafe { as_bytes(b) }.len(), 0);
        // Distinct allocations get distinct starts (we never reserve zero).
        assert_ne!(a as *const u8, b as *const u8);
    }

    #[test]
    fn many_allocations_keep_their_contents() {
        // Far more than one chunk holds, forcing growth, and we re-read *old*
        // pointers afterward to prove nothing moved.
        let arena = Arena::new();
        let mut ptrs = Vec::new();
        for i in 0..5000u32 {
            let s = format!("entry-{i}");
            ptrs.push((arena.alloc_bytes(s.as_bytes()), s));
        }
        for (p, s) in &ptrs {
            assert_eq!(unsafe { as_bytes(*p) }, s.as_bytes());
        }
    }

    #[test]
    fn allocation_larger_than_a_chunk() {
        let arena = Arena::new();
        let big = vec![0xABu8; MAX_CHUNK * 2 + 7];
        let p = arena.alloc_bytes(&big);
        assert_eq!(unsafe { as_bytes(p) }, big.as_slice());
    }

    #[test]
    fn typed_slice_is_aligned_and_correct() {
        let arena = Arena::new();
        let src: Vec<u64> = (0..300).collect();
        let p = arena.alloc_slice(&src);
        assert_eq!(p as *const u64 as usize % core::mem::align_of::<u64>(), 0);
        let got = unsafe { &*p };
        assert_eq!(got, src.as_slice());
    }

    #[test]
    fn concurrent_allocation_is_sound() {
        // Exercises the Sync claim: many threads bump the same arena at once,
        // each verifying it can read back exactly what it wrote.
        let arena = Arc::new(Arena::new());
        let threads: Vec<_> = (0..8u32)
            .map(|t| {
                let arena = Arc::clone(&arena);
                thread::spawn(move || {
                    for i in 0..2000u32 {
                        let s = format!("t{t}-i{i}");
                        let p = arena.alloc_bytes(s.as_bytes());
                        assert_eq!(unsafe { as_bytes(p) }, s.as_bytes());
                    }
                })
            })
            .collect();
        for h in threads {
            h.join().unwrap();
        }
    }
}
