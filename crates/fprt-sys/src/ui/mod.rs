//! UI exports (`fprt_ui_*`) — 167 commands and events across 16 components.
//!
//! Every UI call has the same 5-arg shape; only the payload type (and its
//! direction) varies. These two generic aliases capture that shape, so each
//! export becomes `Pop<P>` (command, engine → host) or `Report<P>` (event,
//! host → engine) with a payload `P` plugged in.

pub mod address_list;
pub mod address_selection;
pub mod application;
pub mod blocked;
pub mod devtools;
pub mod element_type;
pub mod event_tag;
pub mod favorites;
pub mod image_record;
pub mod inputfa;
pub mod inspector;
pub mod language;
pub mod layout_tuple;
pub mod leaptofrogans;
pub mod legalinformation;
pub mod menu;
pub mod pad;
pub mod recentlyvisited;
pub mod recovery;
pub mod sitehandler;
pub mod sld_rect;
pub mod status_name;
pub mod update;
pub mod x_button;
pub mod x_piece;
pub mod x_representation;
pub mod x_rollover;
pub mod zoom;

pub use address_list::AddressList;
pub use address_selection::AddressSelection;
pub use element_type::ElementType;
pub use event_tag::EventTag;
pub use image_record::ImageRecord;
pub use status_name::StatusName;
pub use x_button::XButton;
pub use x_piece::XPiece;
pub use x_representation::XRepresentation;
pub use x_rollover::XRollover;

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
