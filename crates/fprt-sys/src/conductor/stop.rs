//! `fprt_conductor_stop` — tear down a conductor.
//!
//! Export VA `0x6d604350`. Uses the conductor trailing OUT triple
//! (`status3`, `errbuf16`, `mempool_out`); no input payload.

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// `status3`: context handle invalid.
pub const INVALID_CONTEXT: i32 = 0x0bfb_1f99;
/// `status3`: `errbuf16` pointer was NULL.
pub const NULL_ERRBUF: i32 = 0x0bfb_1f9a;
/// `status3`: failed to acquire the context.
pub const ACQUIRE_FAILED: i32 = 0x0bfb_1f9c;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_1f9d;
/// `status3`: conductor in a too-early state (state 10).
pub const WRONG_STATE: i32 = 0x0bfb_1f9e;
/// `status3`: conductor in an otherwise invalid state.
pub const INVALID_STATE: i32 = 0x0bfb_1f9f;
/// `status3`: conductor already stopped (terminal state).
pub const ALREADY_STOPPED: i32 = 0x0bfb_1fa0;
/// `status3`: generic internal failure (alloc / DB clear / unregister).
pub const INTERNAL_FAILURE: i32 = 0x0bfb_1fa2;

/// `int fprt_conductor_stop(ctx, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// Tears the conductor down — clears and deletes its recovery DB and resets it
/// to a terminal state. Returns `1` on success, `0` otherwise; on failure
/// `status3` carries one of the codes above and `errbuf16` a failure ustring.
pub type FprtConductorStop = unsafe extern "C" fn(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
