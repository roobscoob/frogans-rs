//! recovery — Frogans address recovery dialog (`fprt_ui_recovery_*`).
//!
//! 6 commands + 2 events. 4 bare lifecycle (open/show/hide/close — **no push**),
//! `update_labels` (4 strings), `update_addresses` (shared [`AddressList`]),
//! `open` event (shared [`AddressSelection`]), `cancel` bare. Command tags
//! `0x21960a..0x21960f`; command statuses `0x18b0xxxx`, events `0x189f6xxxx`.
//!
//! [`AddressList`]: crate::ui::AddressList
//! [`AddressSelection`]: crate::ui::AddressSelection

pub mod labels;

use crate::ui::{AddressList, AddressSelection, EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x21960a);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x21960b);
pub const CMD_UPDATE_ADDRESSES: StatusName = StatusName(0x21960c);
pub const CMD_SHOW: StatusName = StatusName(0x21960d);
pub const CMD_HIDE: StatusName = StatusName(0x21960e);
pub const CMD_CLOSE: StatusName = StatusName(0x21960f);

// --- event tags ---
pub const EVT_OPEN: EventTag = EventTag(0x10ccf1);
pub const EVT_CANCEL: EventTag = EventTag(0x10ccf2);

// --- the 8 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateAddressesPop = Pop<AddressList>;
pub type OpenReport = Report<AddressSelection>;
pub type CancelReport = Report<EventTag>;
