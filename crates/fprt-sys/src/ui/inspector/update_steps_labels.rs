//! `command_update_steps_labels` payload (`0x20`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The inspector step combobox's entries + the active step.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateStepsLabels {
    pub status_id: StatusName,
    pub reference: i32,
    /// Number of step labels.
    pub count: u32,
    pub _rsv0c: u32,
    /// Mempool array of `count` labels (stride `0x10`).
    pub labels: *const Ustring,
    /// Index to pre-select.
    pub active_step: i32,
    pub _rsv1c: u32,
}
