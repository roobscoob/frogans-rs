//! `language` — the interface-language selection dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::marker_event;

mod cmd_update_labels;
mod cmd_update_list;
mod evt_ok;

pub use cmd_update_labels::UpdateLabels;
pub use cmd_update_list::{Language, UpdateList};
pub use evt_ok::ReportOk;

marker_command!(Open, fprt_sys::ui::language::CMD_OPEN, language_open, LanguageOpen);
marker_command!(Show, fprt_sys::ui::language::CMD_SHOW, language_show, LanguageShow);
marker_command!(Push, fprt_sys::ui::language::CMD_PUSH, language_push, LanguagePush);
marker_command!(Hide, fprt_sys::ui::language::CMD_HIDE, language_hide, LanguageHide);
marker_command!(Close, fprt_sys::ui::language::CMD_CLOSE, language_close, LanguageClose);

marker_event!(ReportCancel, fprt_sys::ui::language::EVT_CANCEL, language_cancel);
