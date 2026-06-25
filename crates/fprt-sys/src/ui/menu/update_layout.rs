//! `command_update_layout` payload (`0x18`) — discarded by the macOS host.

use crate::ui::layout_tuple::LayoutTuple;
use crate::ui::StatusName;

/// Menu geometry tuple. The macOS host discards it (the menu self-positions at
/// the cursor), so the `SldRect` roles are inferred by analogy.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateLayout {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    pub menu_layout: LayoutTuple,
}
