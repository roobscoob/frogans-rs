//! `fprt_conductor_sync_enter` — enter a UI sync/critical region (a turn).
//!
//! Export VA `0x6d603200`. The trailing OUT triple, plus a leading `elapsed_ms`
//! in-param (ms since the last turn; must be ≥ -1).

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// `status3`: context handle invalid.
pub const INVALID_CONTEXT: i32 = 0x0bfb_0ff9;
/// `status3`: `errbuf16` pointer was NULL.
pub const NULL_ERRBUF: i32 = 0x0bfb_0ffa;
/// `status3`: failed to acquire the context.
pub const ACQUIRE_FAILED: i32 = 0x0bfb_0ffc;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_0ffd;
/// `status3`: conductor in the wrong state (internal `0xf918f`).
pub const WRONG_STATE: i32 = 0x0bfb_0ffe;
/// `status3`: already entered — conductor already cruising (internal `0xf9190`).
pub const ALREADY_ENTERED: i32 = 0x0bfb_0fff;
/// `status3`: `elapsed_ms` out of range (must be ≥ -1).
pub const BAD_ELAPSED: i32 = 0x0bfb_1000;
/// `status3`: internal state failure (`0xf9192`).
pub const INTERNAL_STATE: i32 = 0x0bfb_1001;
/// `status3`: generic failure (alloc / object).
pub const INTERNAL_FAILURE: i32 = 0x0bfb_1002;

/// `int fprt_conductor_sync_enter(ctx, int32_t elapsed_ms, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// The UI enters a sync/critical region (a turn), reporting `elapsed_ms` since
/// the last turn (≥ -1). NOTE: unlike the other conductor calls, the dossier
/// found the wrapper returning `0` on all analyzed paths — treat `status3 == 100`
/// as the success signal, not the return.
pub type FprtConductorSyncEnter = unsafe extern "C" fn(
    ctx: Ctx,
    elapsed_ms: i32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
