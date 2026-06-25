//! update — software-update dialog (`fprt_ui_update_*`).
//!
//! 7 commands + 1 event. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (4 strings), `update_data` (2 URIs), `cancel` bare. Command statuses
//! `0x18bfxxxx`, event `0x18bdf04x`.

pub mod labels;
pub mod update_data;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x21962a);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x21962b);
pub const CMD_UPDATE_DATA: StatusName = StatusName(0x21962c);
pub const CMD_SHOW: StatusName = StatusName(0x21962d);
pub const CMD_PUSH: StatusName = StatusName(0x21962e);
pub const CMD_HIDE: StatusName = StatusName(0x21962f);
pub const CMD_CLOSE: StatusName = StatusName(0x219630);

// --- event tag ---
pub const EVT_CANCEL: EventTag = EventTag(0x10ccfe);

// --- the 8 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateDataPop = Pop<update_data::UpdateData>;
pub type CancelReport = Report<EventTag>;
