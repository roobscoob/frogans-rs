//! `command_update_zoom` payload (`0x08`).

use crate::ui::StatusName;

/// Tells the host the current zoom level (integer percent; `100` = 100%).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateZoom {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// Zoom level in percent (host divides by `100.0` for the scale factor).
    pub zoom_level_percent: i32,
}
