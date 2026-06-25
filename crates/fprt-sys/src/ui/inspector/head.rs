//! The shared 8-byte inspector command head (also the lifecycle payload).

use crate::ui::StatusName;

/// The 8-byte head every inspector command payload begins with: an engine status
/// id + the inspector **instance reference** (which window this command targets).
/// For the five lifecycle commands (open/close/show/hide/push) it is the entire
/// payload.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Head {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// Inspector instance reference (engine data_subset id / host `_viewList` key).
    pub reference: i32,
}
