//! `command_update_layout` payload (`0x08`).

use crate::ui::StatusName;

/// Re-emits the application-level layout scalar. The shipping host pops and
/// discards it; no interpreter exists, so the scalar's meaning is **[unresolved]**.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateLayout {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// Application layout value — meaning unresolved.
    pub layout_scalar: u32,
}
