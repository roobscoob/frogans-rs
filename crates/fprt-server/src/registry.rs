//! The mempool-handle registry — the engine side's owned-pool table.
//!
//! The dual of the client's `ForeignPool`. When the engine emits a command it
//! allocates the command's bytes into an [`OwnedPool`], hands the EXE a
//! [`MempoolHandle`] token, and frees the pool when the EXE calls
//! `fprt_library_free_allocated_arguments`. This is the table behind that token:
//! it mints stale-/forge-resistant handles over stored pools and frees each
//! exactly once.
//!
//! It is **instance-owned** — a plain struct with no statics — so every [`Server`]
//! has its own and any number coexist in one process.
//!
//! [`Server`]: crate::Server

use fprt_core::pool::{OwnedPool, Pool};
use fprt_sys::mem::MempoolHandle;

/// A handle packs a slot index in the low [`IDX_BITS`] and a generation counter
/// above it. Reusing a slot bumps its generation, so a reused slot yields a
/// *different* handle and a stale handle is rejected rather than aliased.
const IDX_BITS: u32 = 20;
const IDX_MASK: u32 = (1 << IDX_BITS) - 1;
const GEN_MASK: u32 = u32::MAX >> IDX_BITS;

struct Slot {
    /// Bumped on every (re)use; never `0` once issued, so a never-issued slot
    /// (`generation == 0`) can't be matched by a decoded handle.
    generation: u32,
    /// The pool backing a live handle, or `None` when the slot is free.
    pool: Option<OwnedPool>,
}

/// A per-engine table of live argument pools keyed by [`MempoolHandle`].
pub struct Registry {
    slots: Vec<Slot>,
    free: Vec<usize>,
    live: usize,
}

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Registry {
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
        }
    }

    fn encode(idx: usize, generation: u32) -> MempoolHandle {
        MempoolHandle(((generation & GEN_MASK) << IDX_BITS) | (idx as u32 & IDX_MASK))
    }

    fn decode(h: MempoolHandle) -> (usize, u32) {
        ((h.0 & IDX_MASK) as usize, (h.0 >> IDX_BITS) & GEN_MASK)
    }

    /// Store `pool` and return a fresh handle. Never returns
    /// [`MempoolHandle::EMPTY`].
    pub fn register(&mut self, pool: OwnedPool) -> MempoolHandle {
        let idx = self.free.pop().unwrap_or_else(|| {
            self.slots.push(Slot {
                generation: 0,
                pool: None,
            });
            self.slots.len() - 1
        });
        let slot = &mut self.slots[idx];
        loop {
            slot.generation = slot.generation.wrapping_add(1) & GEN_MASK;
            if slot.generation == 0 {
                continue; // keep it nonzero so "never issued" stays distinguishable
            }
            let handle = Self::encode(idx, slot.generation);
            if handle != MempoolHandle::EMPTY {
                slot.pool = Some(pool);
                self.live += 1;
                return handle;
            }
            // This (idx, generation) collides with the EMPTY sentinel — bump and retry.
        }
    }

    /// Free the pool for `h`, dropping it; returns whether it was live. Stale,
    /// already-freed, forged, or `EMPTY` handles are safe no-ops returning
    /// `false` (matching the engine's documented tolerant `free`).
    pub fn free(&mut self, h: MempoolHandle) -> bool {
        if h == MempoolHandle::EMPTY {
            return false;
        }
        let (idx, generation) = Self::decode(h);
        match self.slots.get_mut(idx) {
            Some(slot) if slot.generation == generation && slot.pool.is_some() => {
                slot.pool = None; // drops the OwnedPool → arena freed on its last ref
                self.free.push(idx);
                self.live -= 1;
                true
            }
            _ => false,
        }
    }

    /// A read token over the pool for `h` — for decoding its bytes back (the
    /// in-process `turn` round-trip), or `None` if `h` isn't live.
    pub fn pool(&self, h: MempoolHandle) -> Option<Pool> {
        let (idx, generation) = Self::decode(h);
        self.slots
            .get(idx)
            .filter(|s| s.generation == generation)
            .and_then(|s| s.pool.as_ref())
            .map(OwnedPool::as_pool)
    }

    /// The number of live (un-freed) handles — for `report_allocated_arguments`.
    pub fn live(&self) -> usize {
        self.live
    }

    /// Total bytes across all live pools — the byte half of
    /// `report_allocated_arguments`.
    pub fn bytes(&self) -> usize {
        self.slots
            .iter()
            .filter_map(|slot| slot.pool.as_ref())
            .map(OwnedPool::bytes)
            .sum()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool_with(s: &str) -> OwnedPool {
        let p = OwnedPool::new();
        let _ = p.alloc_str(s);
        p
    }

    #[test]
    fn register_then_free() {
        let mut r = Registry::new();
        let h = r.register(pool_with("réseau"));
        assert_eq!(r.live(), 1);
        assert!(r.pool(h).is_some());
        assert!(r.free(h));
        assert_eq!(r.live(), 0);
        assert!(r.pool(h).is_none());
    }

    #[test]
    fn double_free_is_a_noop() {
        let mut r = Registry::new();
        let h = r.register(OwnedPool::new());
        assert!(r.free(h));
        assert!(!r.free(h)); // second free: safe no-op
        assert_eq!(r.live(), 0);
    }

    #[test]
    fn stale_handle_after_reuse_is_rejected() {
        let mut r = Registry::new();
        let h1 = r.register(OwnedPool::new());
        assert!(r.free(h1));
        let h2 = r.register(OwnedPool::new()); // reuses the slot with a new generation
        assert_ne!(h1, h2);
        assert!(!r.free(h1)); // the stale handle is rejected
        assert!(r.free(h2));
    }

    #[test]
    fn forged_and_empty_handles_are_noops() {
        let mut r = Registry::new();
        assert!(!r.free(MempoolHandle::EMPTY));
        assert!(!r.free(MempoolHandle(0xDEAD_BEEF)));
        assert!(r.pool(MempoolHandle::EMPTY).is_none());
    }

    #[test]
    fn never_mints_the_empty_sentinel() {
        let mut r = Registry::new();
        for _ in 0..2000 {
            assert_ne!(r.register(OwnedPool::new()), MempoolHandle::EMPTY);
        }
    }

    #[test]
    fn live_handles_are_independent() {
        let mut r = Registry::new();
        let a = r.register(pool_with("a"));
        let b = r.register(pool_with("b"));
        assert_ne!(a, b);
        assert_eq!(r.live(), 2);
        assert!(r.free(a));
        assert!(r.pool(b).is_some()); // freeing `a` doesn't disturb `b`
        assert_eq!(r.live(), 1);
    }

    #[test]
    fn registries_are_independent() {
        // The whole point: instance-owned, no globals.
        let mut r1 = Registry::new();
        let mut r2 = Registry::new();
        let a = r1.register(OwnedPool::new());
        r2.register(OwnedPool::new());
        r2.register(OwnedPool::new());
        assert_eq!(r1.live(), 1);
        assert_eq!(r2.live(), 2);
        assert!(r1.free(a));
        assert_eq!(r1.live(), 0);
        assert_eq!(r2.live(), 2); // r2 is untouched
    }
}
