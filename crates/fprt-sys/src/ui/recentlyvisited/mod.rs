//! recentlyvisited — recently-visited addresses dialog (`fprt_ui_recentlyvisited_*`).
//!
//! 7 commands + 4 events. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (6 strings), `update_addresses` (shared [`AddressList`]), `open`/`delete`
//! events (shared [`AddressSelection`]), `delete_all`/`cancel` bare. Command
//! statuses `0x1854xxxx`, event statuses `0x1853xxxx`.
//!
//! [`AddressList`]: crate::ui::AddressList
//! [`AddressSelection`]: crate::ui::AddressSelection

pub mod labels;

use crate::ui::{AddressList, AddressSelection, EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195de);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195df);
pub const CMD_UPDATE_ADDRESSES: StatusName = StatusName(0x2195e0);
pub const CMD_SHOW: StatusName = StatusName(0x2195e1);
pub const CMD_PUSH: StatusName = StatusName(0x2195e2);
pub const CMD_HIDE: StatusName = StatusName(0x2195e3);
pub const CMD_CLOSE: StatusName = StatusName(0x2195e4);

// --- event tags ---
pub const EVT_OPEN: EventTag = EventTag(0x10cce1);
pub const EVT_DELETE: EventTag = EventTag(0x10cce2);
pub const EVT_DELETE_ALL: EventTag = EventTag(0x10cce3);
pub const EVT_CANCEL: EventTag = EventTag(0x10cce4);

// --- the 11 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateAddressesPop = Pop<AddressList>;
pub type OpenReport = Report<AddressSelection>;
pub type DeleteReport = Report<AddressSelection>;
pub type DeleteAllReport = Report<EventTag>;
pub type CancelReport = Report<EventTag>;
