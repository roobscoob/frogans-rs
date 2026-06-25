//! leaptofrogans — "Leap to Frogans" confirmation dialog (`fprt_ui_leaptofrogans_*`).
//!
//! 7 commands + 5 events. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (two-armed, 7 strings), `update_address` (candidate + compliance flag). The
//! five events (confirm/cancel/block/close/purge) are all bare `Report<EventTag>`.
//! Command statuses `0x1873xxxx`, events `0x1871xxxx`.

pub mod labels;
pub mod update_address;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195ec);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195ed);
pub const CMD_UPDATE_ADDRESS: StatusName = StatusName(0x2195ee);
pub const CMD_SHOW: StatusName = StatusName(0x2195ef);
pub const CMD_PUSH: StatusName = StatusName(0x2195f0);
pub const CMD_HIDE: StatusName = StatusName(0x2195f1);
pub const CMD_CLOSE: StatusName = StatusName(0x2195f2);

// --- event tags ---
pub const EVT_CONFIRM: EventTag = EventTag(0x10cce7);
pub const EVT_CANCEL: EventTag = EventTag(0x10cce8);
pub const EVT_BLOCK: EventTag = EventTag(0x10cce9);
pub const EVT_PURGE: EventTag = EventTag(0x10ccea);
pub const EVT_CLOSE: EventTag = EventTag(0x10cceb);

// --- the 12 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateAddressPop = Pop<update_address::UpdateAddress>;
pub type ConfirmReport = Report<EventTag>;
pub type CancelReport = Report<EventTag>;
pub type BlockReport = Report<EventTag>;
pub type PurgeReport = Report<EventTag>;
pub type CloseReport = Report<EventTag>;
