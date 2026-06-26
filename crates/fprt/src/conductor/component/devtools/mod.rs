//! `devtools` — the developers-directory dialog.

use crate::conductor::command::marker_command;
use crate::conductor::report::{address_selection_event, marker_event};

mod cmd_update_addresses;
mod cmd_update_labels;

pub use cmd_update_addresses::UpdateAddresses;
pub use cmd_update_labels::UpdateLabels;

marker_command!(Open, fprt_sys::ui::devtools::CMD_OPEN, devtools_open, DevtoolsOpen);
marker_command!(Show, fprt_sys::ui::devtools::CMD_SHOW, devtools_show, DevtoolsShow);
marker_command!(Push, fprt_sys::ui::devtools::CMD_PUSH, devtools_push, DevtoolsPush);
marker_command!(Hide, fprt_sys::ui::devtools::CMD_HIDE, devtools_hide, DevtoolsHide);
marker_command!(Close, fprt_sys::ui::devtools::CMD_CLOSE, devtools_close, DevtoolsClose);

address_selection_event!(ReportInspect, fprt_sys::ui::devtools::EVT_INSPECT, devtools_inspect);
marker_event!(ReportCancel, fprt_sys::ui::devtools::EVT_CANCEL, devtools_cancel);
