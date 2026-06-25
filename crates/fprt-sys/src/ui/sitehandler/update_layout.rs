//! `command_update_layout` payload (`0x20`).

use crate::ui::sld_rect::SldRect;
use crate::ui::StatusName;

/// Re-position / zoom the site's native window. Host-applied (the FSI lives in
/// its own window).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateLayout {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// `data_subset` id.
    pub site_id: i32,
    /// `0` => host centers the site & ignores `rect`; else use `rect`.
    pub present_flag: u32,
    pub rect: SldRect,
    /// Zoom / user-size scale level (not pixels).
    pub user_size: i32,
}
