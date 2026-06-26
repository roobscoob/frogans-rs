//! The call envelope shared by every conductor / UI call: turn the trailing OUT
//! triple (`status3`, `errbuf16`, `mempool_out`) into a `Result`.

use std::sync::Arc;

use fprt_sys::mem::MempoolHandle;
use fprt_sys::ustring::Ustring;

use crate::engine::EngineInner;
use crate::error::EngineError;
use crate::pool::Pool;

/// Every call writes `100` to `status3` on success, whatever its `int` return —
/// so this, not the return value, is the success signal.
pub(crate) const SUCCESS: i32 = 100;

/// A zeroed `Ustring` to seed an `errbuf16` out-param before a call.
pub(crate) fn empty_errbuf() -> Ustring {
    Ustring {
        len: 0,
        utf8: core::ptr::null(),
    }
}

/// A `{len, utf8}` descriptor borrowing `s` — valid for the duration of one call.
pub(crate) fn ustring(s: &str) -> Ustring {
    Ustring {
        len: s.len() as i32,
        utf8: s.as_ptr(),
    }
}

/// Run a call and resolve its trailing OUT triple. The closure binds `ctx` and
/// any leading in/out params, receiving only the `(status3, errbuf16, mempool)`
/// pointers; the result is the [`Pool`] (success) or an [`EngineError`].
pub(crate) fn invoke(
    engine: &Arc<EngineInner>,
    call: impl FnOnce(*mut i32, *mut Ustring, *mut MempoolHandle) -> i32,
) -> Result<Pool, EngineError> {
    let mut status = 0i32;
    let mut errbuf = empty_errbuf();
    let mut mempool = MempoolHandle::EMPTY;
    let _ = call(&mut status, &mut errbuf, &mut mempool);
    check(engine, status, errbuf, mempool)
}

/// Resolve a finished call's OUT triple: take ownership of `mempool` as a
/// [`Pool`] (freed on the last drop), then hand it back on success — for the
/// caller to draw pooled outputs from — or build an [`EngineError`] carrying the
/// engine's message.
pub(crate) fn check(
    engine: &Arc<EngineInner>,
    status3: i32,
    errbuf: Ustring,
    mempool: MempoolHandle,
) -> Result<Pool, EngineError> {
    let pool = Pool::new(Arc::clone(engine), mempool);
    if status3 == SUCCESS {
        Ok(pool)
    } else {
        // SAFETY: `errbuf` was written by this very call into `mempool`, which
        // `pool` now owns, so its bytes live for as long as the pool.
        let message = unsafe { pool.string(errbuf) };
        Err(EngineError::new(status3, message))
    }
}
