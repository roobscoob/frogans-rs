//! devtools — developers-directory dialog (`fprt_ui_devtools_*`).
//!
//! 7 commands + 2 events. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (4 strings), `update_addresses` (shared [`AddressList`]), `inspect` event
//! (shared [`AddressSelection`]), `cancel` bare. Command tags are the contiguous
//! `0x219603..0x219609` block.
//!
//! [`AddressList`]: crate::ui::AddressList
//! [`AddressSelection`]: crate::ui::AddressSelection

pub mod labels;

use crate::ui::{AddressList, AddressSelection, EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x219603);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x219604);
pub const CMD_UPDATE_ADDRESSES: StatusName = StatusName(0x219605);
pub const CMD_SHOW: StatusName = StatusName(0x219606);
pub const CMD_PUSH: StatusName = StatusName(0x219607);
pub const CMD_HIDE: StatusName = StatusName(0x219608);
pub const CMD_CLOSE: StatusName = StatusName(0x219609);

// --- event tags ---
pub const EVT_INSPECT: EventTag = EventTag(0x10ccef);
pub const EVT_CANCEL: EventTag = EventTag(0x10ccf0);

// --- the 9 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateAddressesPop = Pop<AddressList>;
pub type InspectReport = Report<AddressSelection>;
pub type CancelReport = Report<EventTag>;
