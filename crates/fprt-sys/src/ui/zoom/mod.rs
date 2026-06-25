//! zoom — zoom-settings dialog (`fprt_ui_zoom_*`).
//!
//! 6 commands + 2 events. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (5 strings), `ok` event (the chosen zoom), `cancel` bare. Command statuses
//! `0x1836xxxx`, events `0x18349xxx`.

pub mod event_ok;
pub mod labels;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195d1);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195d2);
pub const CMD_SHOW: StatusName = StatusName(0x2195d3);
pub const CMD_PUSH: StatusName = StatusName(0x2195d4);
pub const CMD_HIDE: StatusName = StatusName(0x2195d5);
pub const CMD_CLOSE: StatusName = StatusName(0x2195d6);

// --- event tags ---
pub const EVT_OK: EventTag = EventTag(0x10ccdb);
pub const EVT_CANCEL: EventTag = EventTag(0x10ccdc);

// --- the 8 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type OkReport = Report<event_ok::EventOk>;
pub type CancelReport = Report<EventTag>;
