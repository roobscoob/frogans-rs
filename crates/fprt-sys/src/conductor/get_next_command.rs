//! `fprt_conductor_get_next_command` — pop the next engine→host command.
//!
//! Export VA `0x6d6034c0`. A 6-arg accessor: beyond the trailing OUT triple it
//! adds `has_command` (was a command popped?) and `command_id` (its class id).
//!
//! A successful *attempt* returns `1` even when the queue is empty — in that
//! case `has_command` is `0`. The `0x0bfb13xx` codes below are genuine errors
//! (wrong conductor state / bad args), not "queue empty".

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// `status3`: context handle invalid.
pub const INVALID_CONTEXT: i32 = 0x0bfb_13e1;
/// `status3`: `errbuf16` pointer was NULL.
pub const NULL_ERRBUF: i32 = 0x0bfb_13e2;
/// `status3`: failed to acquire the context.
pub const ACQUIRE_FAILED: i32 = 0x0bfb_13e4;
/// `status3`: engine fleet not cruising (library not initialized).
pub const NOT_CRUISING: i32 = 0x0bfb_13e5;
/// `status3`: conductor in the wrong state to pop (internal `0xf91f3`).
pub const WRONG_STATE: i32 = 0x0bfb_13e6;
/// `status3`: conductor not ready / wrong phase (internal `0xf91f4`).
pub const NOT_READY: i32 = 0x0bfb_13e7;
/// `status3`: generic alloc / object failure.
pub const INTERNAL_FAILURE: i32 = 0x0bfb_13ea;
/// `status3`: `has_command` or `command_id` pointer was NULL.
pub const NULL_OUT_ARG: i32 = 0x0bfb_17ca;

/// `int fprt_conductor_get_next_command(ctx, uint32_t *has_command, uint32_t *command_id, int32_t *status3, Ustring *errbuf16, MempoolHandle *mempool_out);`
///
/// Pops the next queued command for the conductor (engine → host). Returns `1`
/// on a successful fetch attempt: `has_command` is then `1` with `command_id`
/// holding the `0x2195xx` command-class id, or `0` if the queue was empty.
/// Returns `0` on error, with one of the codes above in `status3`.
pub type FprtConductorGetNextCommand = unsafe extern "C" fn(
    ctx: Ctx,
    has_command: *mut u32,
    command_id: *mut u32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
