//! `fprt_conductor_sleep_leave` — resume the engine from pause.
//!
//! Export VA `0x6d602f50` (ordinal 3). Bare shape: `ctx` + the trailing OUT
//! triple. Drives the conductor from paused (`0xc`) back to running (`0xd`),
//! deleting the pause DB.

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// `status3`: context handle invalid.
pub const INVALID_CONTEXT: i32 = 0x0bfb_0c11;
/// `status3`: `errbuf16` pointer was NULL.
pub const NULL_ERRBUF: i32 = 0x0bfb_0c12;
/// `status3`: failed to acquire the context.
pub const ACQUIRE_FAILED: i32 = 0x0bfb_0c14;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_0c15;
/// `status3`: conductor not in a pausable/paused state (manager `0xf912b`).
pub const WRONG_STATE: i32 = 0x0bfb_0c16;
/// `status3`: invalid conductor state (manager `0xf912d`).
pub const INVALID_STATE: i32 = 0x0bfb_0c17;
/// `status3`: generic failure (alloc / object / DB delete).
pub const INTERNAL_FAILURE: i32 = 0x0bfb_0c1a;

/// `int fprt_conductor_sleep_leave(ctx, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// Resumes the conductor (paused → running). Returns `1` on success, `0`
/// otherwise with a code above in `status3`.
pub type FprtConductorSleepLeave = unsafe extern "C" fn(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
