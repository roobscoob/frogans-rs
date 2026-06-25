//! `command_update_labels` payload (`0xa8`) — title + nine status/button strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The inspector window's full localized text set (10 strings). Names are the
/// engine DWH-getter symbols (PROVEN, dual-confirmed by the host targets).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    pub status_id: StatusName,
    pub reference: i32,
    pub title: Ustring,
    pub run_completed: Ustring,
    pub run_rejection_raised: Ustring,
    pub synchronize_button: Ustring,
    pub rerun_button_reload: Ustring,
    pub rerun_button_retry: Ustring,
    pub run_data_not_available: Ustring,
    pub autosync_button_on: Ustring,
    pub autosync_button_off: Ustring,
    pub close_button: Ustring,
}
