//! `command_update_layout` payload (`0x18`) — the pad window's layout tuple.

use crate::ui::layout_tuple::LayoutTuple;
use crate::ui::StatusName;

/// The pad window's present-flag + on-screen rectangle (the shared
/// [`LayoutTuple`]). The only data-bearing pad command; on the macOS build the
/// host discards the geometry (structure PROVEN, semantics host-discarded).
///
/// [`LayoutTuple`]: crate::ui::layout_tuple::LayoutTuple
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateLayout {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// +0x04: `present_flag` + `SldRect` (`screen_index`, `reserved`, `x`, `y`).
    pub layout: LayoutTuple,
}
