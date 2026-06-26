//! `update` — the software-update dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::marker_event;

mod cmd_update_data;
mod cmd_update_labels;

pub use cmd_update_data::UpdateData;
pub use cmd_update_labels::UpdateLabels;

marker_command!(Open, fprt_sys::ui::update::CMD_OPEN, update_open, UpdateOpen);
marker_command!(Show, fprt_sys::ui::update::CMD_SHOW, update_show, UpdateShow);
marker_command!(Push, fprt_sys::ui::update::CMD_PUSH, update_push, UpdatePush);
marker_command!(Hide, fprt_sys::ui::update::CMD_HIDE, update_hide, UpdateHide);
marker_command!(Close, fprt_sys::ui::update::CMD_CLOSE, update_close, UpdateClose);

marker_event!(ReportCancel, fprt_sys::ui::update::EVT_CANCEL, update_cancel);
