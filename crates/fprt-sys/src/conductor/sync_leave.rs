//! `fprt_conductor_sync_leave` — leave a UI sync/critical region (end a turn).
//!
//! Export VA `0x6d603db0`. Adds a `*mut u32` OUT (`nextwake_out`, the next-wake
//! delay in ms) before the trailing OUT triple.

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// `status3`: context handle invalid.
pub const INVALID_CONTEXT: i32 = 0x0bfb_17c9;
/// `status3`: `errbuf16` pointer was NULL.
pub const NULL_ERRBUF: i32 = 0x0bfb_17ca;
/// `status3`: failed to acquire the context.
pub const ACQUIRE_FAILED: i32 = 0x0bfb_17cc;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_17cd;
/// `status3`: conductor in the wrong state (internal `0xf9257`).
pub const WRONG_STATE: i32 = 0x0bfb_17ce;
/// `status3`: conductor phase too low to leave (internal `0xf9258`).
pub const PHASE_TOO_LOW: i32 = 0x0bfb_17cf;
/// `status3`: conductor phase too high to leave (internal `0xf9259`).
pub const PHASE_TOO_HIGH: i32 = 0x0bfb_17d0;
/// `status3`: generic failure (ustring / mempool / object).
pub const INTERNAL_FAILURE: i32 = 0x0bfb_17d2;

/// `int fprt_conductor_sync_leave(ctx, uint32_t *nextwake_out, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// Leaves the sync region (ends the turn): on success writes the next-wake delay
/// in ms to `nextwake_out` (`-1` = idle, nothing pending) and returns `1`; `0`
/// otherwise with a code above in `status3`.
pub type FprtConductorSyncLeave = unsafe extern "C" fn(
    ctx: Ctx,
    nextwake_out: *mut u32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
