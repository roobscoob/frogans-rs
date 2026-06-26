//! `leaptofrogans` — the "Leap to Frogans" confirmation dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::marker_event;

mod cmd_update_address;
mod cmd_update_labels;

pub use cmd_update_address::UpdateAddress;
pub use cmd_update_labels::UpdateLabels;

marker_command!(Open, fprt_sys::ui::leaptofrogans::CMD_OPEN, leaptofrogans_open, LeaptofrogansOpen);
marker_command!(Show, fprt_sys::ui::leaptofrogans::CMD_SHOW, leaptofrogans_show, LeaptofrogansShow);
marker_command!(Push, fprt_sys::ui::leaptofrogans::CMD_PUSH, leaptofrogans_push, LeaptofrogansPush);
marker_command!(Hide, fprt_sys::ui::leaptofrogans::CMD_HIDE, leaptofrogans_hide, LeaptofrogansHide);
marker_command!(Close, fprt_sys::ui::leaptofrogans::CMD_CLOSE, leaptofrogans_close, LeaptofrogansClose);

marker_event!(ReportConfirm, fprt_sys::ui::leaptofrogans::EVT_CONFIRM, leaptofrogans_confirm);
marker_event!(ReportCancel, fprt_sys::ui::leaptofrogans::EVT_CANCEL, leaptofrogans_cancel);
marker_event!(ReportBlock, fprt_sys::ui::leaptofrogans::EVT_BLOCK, leaptofrogans_block);
marker_event!(ReportPurge, fprt_sys::ui::leaptofrogans::EVT_PURGE, leaptofrogans_purge);
marker_event!(ReportClose, fprt_sys::ui::leaptofrogans::EVT_CLOSE, leaptofrogans_close_event);
