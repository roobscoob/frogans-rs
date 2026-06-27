//! The client-side pool backing: a foreign engine mempool freed over FFI.
//!
//! This is the one [`Anchor`] that knows about the engine, so it lives here in
//! the client crate rather than in `fprt-core` — only the host knows how to
//! release a mempool handle (`fprt_library_free_allocated_arguments`).

use std::sync::Arc;

use fprt_core::pool::{Anchor, Pool};
use fprt_sys::mem::MempoolHandle;

use crate::engine::EngineInner;

/// An engine-owned mempool handle, released when the last reference drops.
struct ForeignPool {
    engine: Arc<EngineInner>,
    handle: MempoolHandle,
}

impl Drop for ForeignPool {
    fn drop(&mut self) {
        if self.handle != MempoolHandle::EMPTY {
            // SAFETY: the engine (and thus the module) is alive — we hold an
            // `Arc` to it. `free` is documented as a harmless no-op on empty /
            // stale / already-freed handles, so this can only ever do the right
            // thing exactly once.
            unsafe {
                (self.engine.methods().library_free_allocated_arguments)(self.handle);
            }
        }
    }
}

impl Anchor for ForeignPool {}

/// Wrap a freshly-returned `mempool_out` handle as a [`Pool`] that frees it on
/// last drop.
pub(crate) fn foreign(engine: Arc<EngineInner>, handle: MempoolHandle) -> Pool {
    Pool::from_anchor(Arc::new(ForeignPool { engine, handle }))
}
