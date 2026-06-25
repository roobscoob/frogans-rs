//! blocked — blocked Frogans addresses dialog (`fprt_ui_blocked_*`).
//!
//! 7 commands + 3 events. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (5 strings), `update_addresses` (shared [`AddressList`]), `remove` event
//! (shared [`AddressSelection`]), `remove_all`/`cancel` bare. Command tags are
//! the `0x2196XX` block; command statuses `0x1856xxxx`, events `0x1853xxxx`.
//!
//! [`AddressList`]: crate::ui::AddressList
//! [`AddressSelection`]: crate::ui::AddressSelection

pub mod labels;

use crate::ui::{AddressList, AddressSelection, EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x219616);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x219617);
pub const CMD_UPDATE_ADDRESSES: StatusName = StatusName(0x219618);
pub const CMD_SHOW: StatusName = StatusName(0x219619);
pub const CMD_PUSH: StatusName = StatusName(0x21961a);
pub const CMD_HIDE: StatusName = StatusName(0x21961b);
pub const CMD_CLOSE: StatusName = StatusName(0x21961c);

// --- event tags ---
pub const EVT_REMOVE: EventTag = EventTag(0x10ccf5);
pub const EVT_REMOVE_ALL: EventTag = EventTag(0x10ccf6);
pub const EVT_CANCEL: EventTag = EventTag(0x10ccf7);

// --- the 10 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateAddressesPop = Pop<AddressList>;
pub type RemoveReport = Report<AddressSelection>;
pub type RemoveAllReport = Report<EventTag>;
pub type CancelReport = Report<EventTag>;
