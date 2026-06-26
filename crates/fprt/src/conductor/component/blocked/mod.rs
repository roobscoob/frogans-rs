//! `blocked` — the blocked-addresses dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::{address_selection_event, marker_event};

mod cmd_update_addresses;
mod cmd_update_labels;

pub use cmd_update_addresses::UpdateAddresses;
pub use cmd_update_labels::UpdateLabels;

marker_command!(Open, fprt_sys::ui::blocked::CMD_OPEN, blocked_open, BlockedOpen);
marker_command!(Show, fprt_sys::ui::blocked::CMD_SHOW, blocked_show, BlockedShow);
marker_command!(Push, fprt_sys::ui::blocked::CMD_PUSH, blocked_push, BlockedPush);
marker_command!(Hide, fprt_sys::ui::blocked::CMD_HIDE, blocked_hide, BlockedHide);
marker_command!(Close, fprt_sys::ui::blocked::CMD_CLOSE, blocked_close, BlockedClose);

address_selection_event!(ReportRemove, fprt_sys::ui::blocked::EVT_REMOVE, blocked_remove);
marker_event!(ReportRemoveAll, fprt_sys::ui::blocked::EVT_REMOVE_ALL, blocked_remove_all);
marker_event!(ReportCancel, fprt_sys::ui::blocked::EVT_CANCEL, blocked_cancel);
