//! inputfa — input Frogans Address dialog (`fprt_ui_inputfa_*`).
//!
//! 9 commands + 3 events. A text-entry sheet (no list): 6 bare signal commands
//! (incl. `update_error_clear`), `update_address` / `update_error_raise` (one
//! ustring each), `update_labels` (5 strings), `change`/`ok` events (the field
//! text), `cancel` bare. Command statuses `0x1826xxxx`, event statuses `0x1825xxxx`.

pub mod field_text;
pub mod labels;
pub mod update_address;
pub mod update_error_raise;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195c8);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195c9);
pub const CMD_UPDATE_ADDRESS: StatusName = StatusName(0x2195ca);
pub const CMD_UPDATE_ERROR_RAISE: StatusName = StatusName(0x2195cb);
pub const CMD_UPDATE_ERROR_CLEAR: StatusName = StatusName(0x2195cc);
pub const CMD_SHOW: StatusName = StatusName(0x2195cd);
pub const CMD_PUSH: StatusName = StatusName(0x2195ce);
pub const CMD_HIDE: StatusName = StatusName(0x2195cf);
pub const CMD_CLOSE: StatusName = StatusName(0x2195d0);

// --- event tags ---
pub const EVT_CHANGE: EventTag = EventTag(0x10ccd8);
pub const EVT_OK: EventTag = EventTag(0x10ccd9);
pub const EVT_CANCEL: EventTag = EventTag(0x10ccda);

// --- the 12 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateErrorClearPop = Pop<StatusName>;
pub type UpdateAddressPop = Pop<update_address::UpdateAddress>;
pub type UpdateErrorRaisePop = Pop<update_error_raise::UpdateErrorRaise>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type ChangeReport = Report<field_text::FieldText>;
pub type OkReport = Report<field_text::FieldText>;
pub type CancelReport = Report<EventTag>;
