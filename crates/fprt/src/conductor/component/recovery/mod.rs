//! `recovery` — the Frogans-address recovery dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::{address_selection_event, marker_event};

mod cmd_update_addresses;
mod cmd_update_labels;

pub use cmd_update_addresses::UpdateAddresses;
pub use cmd_update_labels::UpdateLabels;

// No `push` command for this dialog.
marker_command!(Open, fprt_sys::ui::recovery::CMD_OPEN, recovery_open, RecoveryOpen);
marker_command!(Show, fprt_sys::ui::recovery::CMD_SHOW, recovery_show, RecoveryShow);
marker_command!(Hide, fprt_sys::ui::recovery::CMD_HIDE, recovery_hide, RecoveryHide);
marker_command!(Close, fprt_sys::ui::recovery::CMD_CLOSE, recovery_close, RecoveryClose);

address_selection_event!(ReportOpen, fprt_sys::ui::recovery::EVT_OPEN, recovery_open_event);
marker_event!(ReportCancel, fprt_sys::ui::recovery::EVT_CANCEL, recovery_cancel);
