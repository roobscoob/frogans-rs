//! `command_update_visual` payload (`0x68`) — the rendered menu.

use crate::ui::{StatusName, XButton, XRepresentation};

/// The menu's rendered SLD representation (background + rollover regions) plus
/// the interactive entry/button array.
///
/// `update_visual` returns live mempool data — free the image/plane/PNG/label
/// buffers via the call's `mempool_out`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UpdateVisual {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// `0xfa1` global (pad) menu / `0xfa2` site menu / `800` none. [name inferred]
    pub variant: u32,
    /// Site id (written only when `variant == 0xfa2`, else 0).
    pub site_id: u32,
    // +0x0c: implicit pad → representation aligns to +0x10.
    pub representation: XRepresentation,
    /// Number of interactive menu entries.
    pub xbutton_count: i32,
    // +0x5c: implicit pad → xbuttons aligns to +0x60.
    /// Mempool array of `xbutton_count` buttons (stride `0x38`).
    pub xbuttons: *mut XButton,
}
