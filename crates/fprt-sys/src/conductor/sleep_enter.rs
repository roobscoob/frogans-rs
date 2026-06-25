//! `fprt_conductor_sleep_enter` — pause the engine (host idle / backgrounded).
//!
//! Export VA `0x6d6040a0`. Bare shape: `ctx` + the trailing OUT triple. Drives
//! the conductor from running (state `0xd`) to paused (`0xc`), saving the pause DB.

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// `status3`: context handle invalid.
pub const INVALID_CONTEXT: i32 = 0x0bfb_1bb1;
/// `status3`: `errbuf16` pointer was NULL.
pub const NULL_ERRBUF: i32 = 0x0bfb_1bb2;
/// `status3`: failed to acquire the context.
pub const ACQUIRE_FAILED: i32 = 0x0bfb_1bb4;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_1bb5;
/// `status3`: conductor in the wrong state (manager `0xf92bb`).
pub const WRONG_STATE: i32 = 0x0bfb_1bb6;
/// `status3`: conductor already paused (manager `0xf92bd`).
pub const ALREADY_PAUSED: i32 = 0x0bfb_1bb8;
/// `status3`: generic failure (alloc / object / DB save).
pub const INTERNAL_FAILURE: i32 = 0x0bfb_1bba;

/// `int fprt_conductor_sleep_enter(ctx, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// Pauses the conductor (running → paused). Returns `1` on success, `0`
/// otherwise with a code above in `status3`.
pub type FprtConductorSleepEnter = unsafe extern "C" fn(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
