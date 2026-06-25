//! language — interface-language selection dialog (`fprt_ui_language_*`).
//!
//! 7 commands + 2 events. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (5 strings), `update_list` (the selectable-language list), `ok` event (the
//! chosen language id), `cancel` bare. Command statuses `0x18456xxx`, events
//! `0x1843xxxx`.

pub mod event_ok;
pub mod labels;
pub mod update_list;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195e5);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195e6);
pub const CMD_UPDATE_LIST: StatusName = StatusName(0x2195e7);
pub const CMD_SHOW: StatusName = StatusName(0x2195e8);
pub const CMD_PUSH: StatusName = StatusName(0x2195e9);
pub const CMD_HIDE: StatusName = StatusName(0x2195ea);
pub const CMD_CLOSE: StatusName = StatusName(0x2195eb);

// --- event tags ---
pub const EVT_OK: EventTag = EventTag(0x10cce5);
pub const EVT_CANCEL: EventTag = EventTag(0x10cce6);

// --- the 9 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateListPop = Pop<update_list::UpdateList>;
pub type OkReport = Report<event_ok::LanguageOk>;
pub type CancelReport = Report<EventTag>;
