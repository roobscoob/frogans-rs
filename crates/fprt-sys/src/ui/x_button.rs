//! `XButton` — one interactive zone element (`0x38`, the wire form).

use crate::ui::{ElementType, ImageRecord};
use crate::ustring::Ustring;

/// One entry of an `update_visual` button array — a clickable zone descriptor.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XButton {
    /// Zone element-type (base `0x6b724`).
    pub element_type: ElementType,
    // +0x04: implicit pad → label aligns to +0x08.
    /// Entry / address text.
    pub label: Ustring,
    /// `1` => entry text concealed (password-style).
    pub concealed: u32,
    /// Entry common-data state (exact enum unresolved).
    pub entry_state: i32,
    /// `frogans_site` icon image. Left zero by the engine on this build (the host
    /// reads it for `ElementType::FROGANS_SITE`, guarding on a non-zero buffer).
    pub icon_image: ImageRecord,
}
