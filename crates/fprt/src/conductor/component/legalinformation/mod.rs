//! `legalinformation` — the legal-information / OSS-license panel.

use crate::conductor::command::marker_command;
use crate::conductor::report::marker_event;

mod cmd_update_labels;
mod cmd_update_legal_content;

pub use cmd_update_labels::UpdateLabels;
pub use cmd_update_legal_content::{Document, LegalContentKind, Topic, UpdateLegalContent};

marker_command!(Open, fprt_sys::ui::legalinformation::CMD_OPEN, legalinformation_open, LegalinformationOpen);
marker_command!(Show, fprt_sys::ui::legalinformation::CMD_SHOW, legalinformation_show, LegalinformationShow);
marker_command!(Push, fprt_sys::ui::legalinformation::CMD_PUSH, legalinformation_push, LegalinformationPush);
marker_command!(Hide, fprt_sys::ui::legalinformation::CMD_HIDE, legalinformation_hide, LegalinformationHide);
marker_command!(Close, fprt_sys::ui::legalinformation::CMD_CLOSE, legalinformation_close, LegalinformationClose);

marker_event!(ReportClose, fprt_sys::ui::legalinformation::EVT_CLOSE, legalinformation_close_event);
