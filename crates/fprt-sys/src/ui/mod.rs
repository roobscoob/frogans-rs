//! UI exports (`fprt_ui_*`) — 167 commands and events across 16 components.
//!
//! Every UI call has the same 5-arg shape; only the payload type (and its
//! direction) varies. These two generic aliases capture that shape, so each
//! export becomes `Pop<P>` (command, engine → host) or `Report<P>` (event,
//! host → engine) with a payload `P` plugged in.

use crate::ctx::Ctx;
use crate::mem::MempoolHandle;
use crate::ustring::Ustring;

/// A command export (`_pop`, engine → host): the engine writes the **OUT**
/// payload `*mut P` (field 0 = the engine-written status-name id).
pub type Pop<P> = unsafe extern "C" fn(
    ctx: Ctx,
    payload: *mut P,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;

/// An event export (`_report`, host → engine): the host supplies the **IN**
/// payload `*const P` (field 0 = the host-written event tag).
pub type Report<P> = unsafe extern "C" fn(
    ctx: Ctx,
    payload: *const P,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32;
