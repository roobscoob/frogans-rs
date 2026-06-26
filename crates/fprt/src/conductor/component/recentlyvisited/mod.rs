//! `recentlyvisited` — the recently-visited-addresses dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::{address_selection_event, marker_event};

mod cmd_update_addresses;
mod cmd_update_labels;

pub use cmd_update_addresses::UpdateAddresses;
pub use cmd_update_labels::UpdateLabels;

marker_command!(Open, fprt_sys::ui::recentlyvisited::CMD_OPEN, recentlyvisited_open, RecentlyvisitedOpen);
marker_command!(Show, fprt_sys::ui::recentlyvisited::CMD_SHOW, recentlyvisited_show, RecentlyvisitedShow);
marker_command!(Push, fprt_sys::ui::recentlyvisited::CMD_PUSH, recentlyvisited_push, RecentlyvisitedPush);
marker_command!(Hide, fprt_sys::ui::recentlyvisited::CMD_HIDE, recentlyvisited_hide, RecentlyvisitedHide);
marker_command!(Close, fprt_sys::ui::recentlyvisited::CMD_CLOSE, recentlyvisited_close, RecentlyvisitedClose);

address_selection_event!(ReportOpen, fprt_sys::ui::recentlyvisited::EVT_OPEN, recentlyvisited_open_event);
address_selection_event!(ReportDelete, fprt_sys::ui::recentlyvisited::EVT_DELETE, recentlyvisited_delete);
marker_event!(ReportDeleteAll, fprt_sys::ui::recentlyvisited::EVT_DELETE_ALL, recentlyvisited_delete_all);
marker_event!(ReportCancel, fprt_sys::ui::recentlyvisited::EVT_CANCEL, recentlyvisited_cancel);
