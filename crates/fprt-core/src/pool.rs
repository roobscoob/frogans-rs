//! The refcounted argument-memory primitive.
//!
//! Every FPRT call hands back argument data — strings, image blobs, diagnostic
//! messages — in a pool the *producing* side owns and the *consuming* side reads.
//! A [`Pool`] is the shared owner of one such region: it frees on last drop. A
//! [`Pooled<T>`] is a raw pointer *into* that region plus a retained [`Pool`] — a
//! refcount-freed `Box<T>` whose backing allocation belongs to the pool.
//!
//! What backs a pool is abstracted by [`Anchor`]: the caller (client) backs it
//! with a foreign engine mempool handle freed over FFI; the implementor (server)
//! backs it with an owned [`Arena`] via [`OwnedPool`]. `Pooled` is identical
//! either way.
//!
//! Safe, typed wrappers ([`PooledString`], [`PooledImage`]) are newtypes over
//! `Pooled<[u8]>`; end users never touch the one `unsafe` choke point,
//! [`Pool::own`].

#![allow(dead_code)]

use core::ops::Deref;
use core::str::Utf8Error;
use std::borrow::Cow;
use std::sync::Arc;

use fprt_sys::ui::ImageRecord;
use fprt_sys::ustring::Ustring;

use crate::arena::Arena;

/// Keeps a region of argument bytes alive; the last [`Pool`] clone to drop frees
/// it. Pure RAII — no methods. The concrete implementor's `Drop` does the
/// freeing, and the `Send + Sync` bound is what lets a [`Pooled`] cross threads.
///
/// The two implementors are the two sides of the ABI: a *foreign* pool (the
/// client borrows an engine-owned mempool and releases it over FFI — it lives in
/// the client crate, since only it knows how to free) and [`Arena`] (the server
/// owns the bytes outright and frees them on drop). A [`Pooled`] holds one
/// type-erased behind `Arc<dyn Anchor>` and never needs to know which.
pub trait Anchor: Send + Sync {}

/// A shared-ownership token over one argument region. Cheap to clone; the region
/// is freed when the last clone (across all [`Pooled`] minted from it) drops.
///
/// Backed by either a foreign engine mempool (built in the client crate via
/// [`Pool::from_anchor`]) or an owned [`Arena`] ([`OwnedPool`]) — the same
/// [`Pooled`] views either way.
#[derive(Clone)]
pub struct Pool(Arc<dyn Anchor>);

impl Pool {
    /// Wrap any [`Anchor`] as a pool. The client uses this to install its
    /// foreign-mempool backing; [`OwnedPool`] uses it for the arena backing.
    pub fn from_anchor(anchor: Arc<dyn Anchor>) -> Self {
        Pool(anchor)
    }

    /// Mint a [`Pooled<T>`] pointing at `ptr`, sharing this pool's refcount.
    ///
    /// # Safety
    ///
    /// `ptr` must point into this pool's memory at a valid `T` that stays valid
    /// for as long as the pool is alive. Callers reach this only through safe
    /// wrappers ([`Pool::string`], typed payload accessors).
    pub unsafe fn own<T: ?Sized>(&self, ptr: *const T) -> Pooled<T> {
        Pooled {
            pool: self.clone(),
            ptr,
        }
    }

    /// Interpret an engine `Ustring` as a pooled byte string, or `None` if the
    /// descriptor is null / empty.
    ///
    /// # Safety
    ///
    /// `raw` must be a descriptor the engine wrote into *this* pool (e.g. this
    /// call's `errbuf16`, or a string field of a payload it returned here), so
    /// its bytes live for as long as the pool. The null/empty check is a
    /// convenience, not a provenance check — a non-null pointer from elsewhere
    /// would still be unsound.
    pub unsafe fn string(&self, raw: Ustring) -> Option<PooledString> {
        if raw.utf8.is_null() || raw.len <= 0 {
            return None;
        }
        let bytes: *const [u8] = core::ptr::slice_from_raw_parts(raw.utf8, raw.len as usize);
        // SAFETY: by this fn's contract `raw` belongs to this pool, so `bytes`
        // points at `raw.len` valid bytes that live until the pool is freed,
        // which our retained clone defers.
        Some(PooledString(unsafe { self.own(bytes) }))
    }

    /// Interpret an engine `ImageRecord` as a pooled image, or `None` if it
    /// carries no bytes.
    ///
    /// # Safety
    ///
    /// `raw` must be an `ImageRecord` the engine wrote into *this* pool (same
    /// provenance caveat as [`string`](Pool::string)).
    pub unsafe fn image(&self, raw: ImageRecord) -> Option<PooledImage> {
        if raw.buffer.is_null() || raw.byte_len == 0 {
            return None;
        }
        let bytes: *const [u8] = core::ptr::slice_from_raw_parts(raw.buffer, raw.byte_len as usize);
        // SAFETY: by this fn's contract `raw` belongs to this pool, so `bytes`
        // points at `byte_len` valid bytes living as long as the pool; and the
        // host reads the `dim` union as the `{ width, height }` it overlays.
        let (bytes, size) = unsafe { (self.own(bytes), raw.dim.size) };
        Some(PooledImage {
            bytes,
            width: size.width,
            height: size.height,
        })
    }

    /// Read a mempool array of `count` `Ustring`s into pooled strings (null/empty
    /// entries are dropped). For the OUT address / label lists.
    ///
    /// # Safety
    ///
    /// `items` must point at `count` `Ustring`s the engine wrote into *this* pool.
    pub unsafe fn strings(&self, items: *const Ustring, count: u32) -> Vec<PooledString> {
        if items.is_null() {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            // SAFETY: `items` has `count` entries, each written into this pool.
            let raw = unsafe { *items.add(i) };
            if let Some(s) = unsafe { self.string(raw) } {
                out.push(s);
            }
        }
        out
    }
}

/// The owned-allocation side: an [`Arena`] you copy data *into*, minting
/// [`Pooled`] views that share its lifetime. The dual of [`Pool`]'s foreign read
/// path — this is what the engine side uses to build the data it hands back, and
/// it works freestanding (no engine, no DLL) too.
///
/// Cloning an `OwnedPool` shares the same arena, so values minted from any clone
/// keep the whole region alive until the last [`Pooled`] drops.
#[derive(Clone)]
pub struct OwnedPool {
    arena: Arc<Arena>,
}

impl OwnedPool {
    /// A fresh, empty arena.
    pub fn new() -> Self {
        OwnedPool {
            arena: Arc::new(Arena::new()),
        }
    }

    // (`Default` is provided below so the pool reads as a normal allocator.)

    /// The shared [`Pool`] read-token backing every value minted here — for
    /// threading the same arena through a payload's nested fields, and for
    /// decoding bytes the arena owns back into [`Pooled`] views.
    pub fn as_pool(&self) -> Pool {
        Pool(self.arena.clone())
    }

    /// Total bytes this pool has allocated — its contribution to
    /// `library_report_allocated_arguments`' byte total.
    pub fn bytes(&self) -> usize {
        self.arena.allocated()
    }

    /// Copy `s` into the arena and hand back an owned [`PooledString`].
    pub fn alloc_str(&self, s: &str) -> PooledString {
        let ptr = self.arena.alloc_bytes(s.as_bytes());
        // SAFETY: `ptr` points at `s.len()` bytes just copied into this arena,
        // which the retained `as_pool()` keeps alive; they're immutable thereafter.
        PooledString(unsafe { self.as_pool().own(ptr) })
    }

    /// Copy a `T` slice into the arena, returning a stable pointer to it — for
    /// the descriptor arrays (`[Ustring]` / `[ImageRecord]`) a payload points at.
    pub fn alloc_slice<T: Copy>(&self, src: &[T]) -> *const [T] {
        self.arena.alloc_slice(src)
    }

    /// Copy encoded image `bytes` (with their pixel dimensions) into the arena and
    /// hand back an owned [`PooledImage`].
    pub fn alloc_image(&self, bytes: &[u8], width: u32, height: u32) -> PooledImage {
        let ptr = self.arena.alloc_bytes(bytes);
        // SAFETY: `ptr` points at `bytes.len()` bytes just copied into this arena,
        // kept alive by the retained `as_pool()`; immutable thereafter.
        PooledImage {
            bytes: unsafe { self.as_pool().own(ptr) },
            width,
            height,
        }
    }

    /// Copy a [`PooledString`] from *any* pool into this one (byte-faithful — copies
    /// the raw bytes, so it survives even non-UTF-8 content). For deep-copying a
    /// payload out of a foreign pool into an owned one.
    pub fn clone_str(&self, s: &PooledString) -> PooledString {
        let ptr = self.arena.alloc_bytes(s.as_bytes());
        // SAFETY: `ptr` points at the bytes just copied into this arena, kept alive
        // by the retained `as_pool()`; immutable thereafter.
        PooledString(unsafe { self.as_pool().own(ptr) })
    }

    /// Copy an optional [`PooledString`] into this pool (see [`clone_str`](Self::clone_str)).
    pub fn clone_str_opt(&self, s: &Option<PooledString>) -> Option<PooledString> {
        s.as_ref().map(|s| self.clone_str(s))
    }

    /// Copy a [`PooledImage`] from *any* pool into this one.
    pub fn clone_image(&self, image: &PooledImage) -> PooledImage {
        self.alloc_image(image.bytes(), image.width(), image.height())
    }

    /// Copy an optional [`PooledImage`] into this pool (see [`clone_image`](Self::clone_image)).
    pub fn clone_image_opt(&self, image: &Option<PooledImage>) -> Option<PooledImage> {
        image.as_ref().map(|i| self.clone_image(i))
    }
}

impl Default for OwnedPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A `T` that lives in engine pool memory: a raw pointer into the pool plus a
/// retained [`Pool`] keeping it alive. Like `Box<T>`, but freed by refcount.
///
/// Holding one pins the engine from `finalize`ing — exactly so the data stays
/// readable for as long as you keep it. Deref borrows are tied to `&self`, so a
/// `&T` can never outlive the pool.
pub struct Pooled<T: ?Sized> {
    pool: Pool,
    ptr: *const T,
}

impl<T: ?Sized> Pooled<T> {
    /// The pool backing this value — for minting sibling views into the same
    /// allocation (e.g. the inner strings of a payload struct).
    pub fn pool(&self) -> &Pool {
        &self.pool
    }
}

impl<T: ?Sized> Deref for Pooled<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: `ptr` points at a valid `T` (per `own`'s contract) and the
        // backing memory is alive because `self.pool` is. The returned borrow is
        // tied to `&self`, so it cannot outlive the pool.
        unsafe { &*self.ptr }
    }
}

impl<T: ?Sized> Clone for Pooled<T> {
    fn clone(&self) -> Self {
        Pooled {
            pool: self.pool.clone(),
            ptr: self.ptr,
        }
    }
}

// SAFETY: a `Pooled<T>` is a shared, refcounted pointer into *immutable* engine
// memory kept alive by its `Arc`'d `Pool`. We only ever read `T` (through
// `Deref`) and never relocate or run a destructor for a `T` — dropping the last
// reference frees raw bytes via `free_allocated_arguments`, which is fully
// thread-safe. So the only cross-thread hazard is concurrent reads of `T`, which
// `T: Sync` rules out. Note this is *weaker* than `Arc<T>`'s `T: Send + Sync`:
// we require no `T: Send`, because no `T` is ever moved or dropped across threads
// — the bytes live and die in the engine's pool, not Rust's heap.
unsafe impl<T: ?Sized + Sync> Send for Pooled<T> {}
unsafe impl<T: ?Sized + Sync> Sync for Pooled<T> {}

// PooledMut<T> (a `*mut T` variant with `DerefMut`) goes here when an actual
// host-writable engine buffer appears. It carries a no-aliasing obligation, so
// we don't build it speculatively.

/// Engine-owned bytes interpreted as a UTF-8 string.
///
/// The engine *claims* UTF-8; [`as_str`](PooledString::as_str) verifies it and
/// can fail, while [`to_string_lossy`](PooledString::to_string_lossy) is the
/// infallible escape.
pub struct PooledString(Pooled<[u8]>);

impl PooledString {
    /// The raw engine bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The string, if the engine's bytes are valid UTF-8.
    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(&self.0)
    }

    /// The string, with any invalid UTF-8 replaced by U+FFFD.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }
}

impl core::fmt::Debug for PooledString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Show it as a (lossily-decoded) string, quoted.
        core::fmt::Debug::fmt(&self.to_string_lossy(), f)
    }
}

/// An engine-encoded image: a zero-copy view of its bytes plus the dimensions
/// the engine reports.
///
/// The bytes are in whatever pixel format the conductor's
/// [`ImageFormat`](crate::ImageFormat) selected — PNG-encoded, or raw pixels.
pub struct PooledImage {
    bytes: Pooled<[u8]>,
    width: u32,
    height: u32,
}

impl PooledImage {
    /// The encoded image bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Image width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Image height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl core::fmt::Debug for PooledImage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.bytes.len();
        f.debug_struct("PooledImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes", &format_args!("{len} bytes"))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_pool_mints_readable_string() {
        let pool = OwnedPool::new();
        let s = pool.alloc_str("réseau");
        assert_eq!(s.as_str().unwrap(), "réseau");
        assert_eq!(s.as_bytes(), "réseau".as_bytes());
    }

    #[test]
    fn pooled_string_outlives_the_owned_pool_handle() {
        // The `OwnedPool` handle drops at the end of the block, but the value
        // keeps the arena alive through its retained `Pool` — proving the
        // refcount-frees-on-last-drop semantics hold for the owned backing too.
        let s = {
            let pool = OwnedPool::new();
            pool.alloc_str("persists")
        };
        assert_eq!(s.as_str().unwrap(), "persists");
    }

    #[test]
    fn many_strings_from_one_pool_share_the_arena() {
        let pool = OwnedPool::new();
        let values: Vec<PooledString> = (0..1000).map(|i| pool.alloc_str(&format!("addr-{i}"))).collect();
        for (i, s) in values.iter().enumerate() {
            assert_eq!(s.as_str().unwrap(), format!("addr-{i}"));
        }
    }
}
