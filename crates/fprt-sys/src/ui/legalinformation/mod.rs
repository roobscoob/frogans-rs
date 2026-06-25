//! legalinformation — legal-information / OSS-license panel (`fprt_ui_legalinformation_*`).
//!
//! 7 commands + 1 event. 5 bare lifecycle (`Pop<StatusName>`), `update_labels`
//! (5 strings), `update_legal_content` (content-kind + optional image + a nested
//! document tree), `close` event bare. Command statuses `0x1817xxxx`, event
//! `0x18161bxx`.

pub mod labels;
pub mod legal_content;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195f3);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x2195f4);
pub const CMD_UPDATE_LEGAL_CONTENT: StatusName = StatusName(0x2195f5);
pub const CMD_SHOW: StatusName = StatusName(0x2195f6);
pub const CMD_PUSH: StatusName = StatusName(0x2195f7);
pub const CMD_HIDE: StatusName = StatusName(0x2195f8);
pub const CMD_CLOSE: StatusName = StatusName(0x2195f9);

// --- event tag ---
pub const EVT_CLOSE: EventTag = EventTag(0x10ccec);

// --- the 8 calls ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateLegalContentPop = Pop<legal_content::UpdateLegalContent>;
pub type CloseReport = Report<EventTag>;
