//! `command_update_status` payload (`0x10`).

use crate::ui::StatusName;

/// The inspector's run status + a run-data-available flag.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateStatus {
    pub status_id: StatusName,
    pub reference: i32,
    /// `0x3e9` run completed, `0x3ea` run rejection raised.
    pub run_status: u32,
    /// 0 ⇒ show "run data not available".
    pub run_data_available: u32,
}
