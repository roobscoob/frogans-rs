//! The `LayoutTuple` present-flag + rect (`0x14`).

use crate::ui::sld_rect::SldRect;

/// A presence flag plus a screen rect (pad / menu layout element).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LayoutTuple {
    /// `0` = no rect supplied.
    pub present_flag: u32,
    pub rect: SldRect,
}
