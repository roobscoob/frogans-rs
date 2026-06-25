//! `command_update_visual` payload (`0xb0`) — the rendered-content carrier.

use crate::ui::{StatusName, XButton, XRepresentation};

/// The site's rendered visual scheme: the vignette + lead slides (RGBA images
/// with interactive rollover regions) and the per-element button/zone list.
///
/// `update_visual` returns live mempool data — free the image/rollover/piece/
/// button buffers via the call's `mempool_out`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UpdateVisual {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// `data_subset` id.
    pub site_id: i32,
    /// Vignette slide.
    pub vignette: XRepresentation,
    /// Lead slide.
    pub lead: XRepresentation,
    /// Number of zone elements.
    pub button_count: u32,
    // +0x9c: implicit pad → buttons aligns to +0xa0.
    /// Mempool array of `button_count` buttons (stride `0x38`).
    pub buttons: *mut XButton,
    /// Tail of the `0xb0` memset span (no field written).
    pub _rsv_a8: u64,
}
