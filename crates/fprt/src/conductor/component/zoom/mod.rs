//! `zoom` — the zoom-settings dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::marker_event;

mod cmd_update_labels;
mod evt_ok;

pub use cmd_update_labels::UpdateLabels;
pub use evt_ok::ReportOk;

marker_command!(Open, fprt_sys::ui::zoom::CMD_OPEN, zoom_open, ZoomOpen);
marker_command!(Show, fprt_sys::ui::zoom::CMD_SHOW, zoom_show, ZoomShow);
marker_command!(Push, fprt_sys::ui::zoom::CMD_PUSH, zoom_push, ZoomPush);
marker_command!(Hide, fprt_sys::ui::zoom::CMD_HIDE, zoom_hide, ZoomHide);
marker_command!(Close, fprt_sys::ui::zoom::CMD_CLOSE, zoom_close, ZoomClose);

marker_event!(ReportCancel, fprt_sys::ui::zoom::EVT_CANCEL, zoom_cancel);
