//! The shared engine handle that every other type hangs off.
//!
//! [`EngineInner`] owns the host (and thus the mapped module). It is held behind
//! an [`Arc`](std::sync::Arc): [`Library`](crate::Library) holds one, and so does
//! every [`Pool`](crate::pool::Pool) and every message-bearing
//! [`EngineError`](crate::EngineError). The engine only `finalize`s when the
//! *last* of those drops — so engine-owned data (an error message, a pooled
//! string) is always safe to read for as long as you hold it.
//!
//! Multithreaded: [`FprtHost`] is `Send + Sync`, so `EngineInner` and every
//! `Arc` of it cross threads freely. The one exception is the init/finalize
//! transition, which the engine does *not* synchronize internally — the
//! [`LIFECYCLE`] lock serializes those.

use std::sync::{Mutex, MutexGuard};

use fprt_sys::Fprt;

use crate::host::FprtHost;

/// Serializes `library_initialize` / `library_finalize` across the process.
///
/// These transitions are not internally synchronized, and two hosts can share
/// one module instance (same DLL → same `HMODULE` → same global init state), so
/// every init and finalize must be mutually exclusive. This is a *critical
/// section*, not a singleton flag: it never rejects a host, it only orders the
/// calls — so multiple independent engines coexist fine. (Single-init ownership
/// is enforced separately, by the engine returning `ALREADY_INITIALIZED`.)
static LIFECYCLE: Mutex<()> = Mutex::new(());

/// Acquire the lifecycle lock for the duration of one init or finalize call.
///
/// The guarded data is `()`, so a poisoned lock carries no broken invariant —
/// we recover from poisoning rather than propagate it.
pub(crate) fn lifecycle_lock() -> MutexGuard<'static, ()> {
    LIFECYCLE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The live engine: the host (mapped module + export table) plus the obligation
/// to `finalize` on teardown. Shared via `Arc`; see the module docs.
pub(crate) struct EngineInner {
    host: Box<dyn FprtHost>,
}

impl EngineInner {
    pub(crate) fn new(host: Box<dyn FprtHost>) -> Self {
        EngineInner { host }
    }

    /// The export table. Valid for as long as `self` lives (the host keeps the
    /// module mapped).
    pub(crate) fn methods(&self) -> &Fprt {
        self.host.methods()
    }
}

impl Drop for EngineInner {
    fn drop(&mut self) {
        let mut status: u32 = 0;
        // Serialize against any concurrent init/finalize. Held only across the
        // call; the module unmaps after this body (when `self.host` drops), which
        // is OS-synchronized and needs no lifecycle lock.
        let _guard = lifecycle_lock();
        // SAFETY: the host (table + module) is still alive — it drops after this
        // body. `finalize` is the reverse teardown; nothing to recover at drop.
        unsafe {
            let _ = (self.host.methods().library_finalize)(&mut status);
        }
    }
}
