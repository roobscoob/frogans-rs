//! `inputfa` — the input-Frogans-Address text-entry dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::marker_event;

mod cmd_update_address;
mod cmd_update_error_raise;
mod cmd_update_labels;
mod evt_change;
mod evt_ok;

pub use cmd_update_address::UpdateAddress;
pub use cmd_update_error_raise::UpdateErrorRaise;
pub use cmd_update_labels::UpdateLabels;
pub use evt_change::ReportChange;
pub use evt_ok::ReportOk;

marker_command!(Open, fprt_sys::ui::inputfa::CMD_OPEN, inputfa_open, InputfaOpen);
marker_command!(Show, fprt_sys::ui::inputfa::CMD_SHOW, inputfa_show, InputfaShow);
marker_command!(Push, fprt_sys::ui::inputfa::CMD_PUSH, inputfa_push, InputfaPush);
marker_command!(Hide, fprt_sys::ui::inputfa::CMD_HIDE, inputfa_hide, InputfaHide);
marker_command!(Close, fprt_sys::ui::inputfa::CMD_CLOSE, inputfa_close, InputfaClose);
marker_command!(
    UpdateErrorClear,
    fprt_sys::ui::inputfa::CMD_UPDATE_ERROR_CLEAR,
    inputfa_update_error_clear,
    InputfaUpdateErrorClear
);

marker_event!(ReportCancel, fprt_sys::ui::inputfa::EVT_CANCEL, inputfa_cancel);
