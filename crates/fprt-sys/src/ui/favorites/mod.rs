//! favorites — favorites manager dialog (`fprt_ui_favorites_*`).
//!
//! 7 commands + 4 events. Structurally the twin of `recentlyvisited`: 5 bare
//! lifecycle (`Pop<StatusName>`), `update_labels` (6 strings), `update_addresses`
//! (shared [`AddressList`]), `open`/`remove` events (shared [`AddressSelection`]),
//! `remove_all`/`cancel` bare.
//!
//! Command-id values are from the architecture appendix; favorites lacks its own
//! consolidated datatypes doc (see [`labels`]).
//!
//! [`AddressList`]: crate::ui::AddressList
//! [`AddressSelection`]: crate::ui::AddressSelection

pub mod labels;

use crate::ui::{AddressList, AddressSelection, EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195d7);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195d8);
pub const CMD_UPDATE_ADDRESSES: StatusName = StatusName(0x2195d9);
pub const CMD_SHOW: StatusName = StatusName(0x2195da);
pub const CMD_PUSH: StatusName = StatusName(0x2195db);
pub const CMD_HIDE: StatusName = StatusName(0x2195dc);
pub const CMD_CLOSE: StatusName = StatusName(0x2195dd);

// --- event tags ---
pub const EVT_OPEN: EventTag = EventTag(0x10ccdd);
pub const EVT_REMOVE: EventTag = EventTag(0x10ccde);
pub const EVT_REMOVE_ALL: EventTag = EventTag(0x10ccdf);
pub const EVT_CANCEL: EventTag = EventTag(0x10cce0);

// --- the 11 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateAddressesPop = Pop<AddressList>;
pub type OpenReport = Report<AddressSelection>;
pub type RemoveReport = Report<AddressSelection>;
pub type RemoveAllReport = Report<EventTag>;
pub type CancelReport = Report<EventTag>;
