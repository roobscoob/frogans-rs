//! `XPiece` — one rollover-state image fragment (`0x48`).

use crate::ui::ImageRecord;

/// One image fragment of an [`XRollover`]: a geometry, a raw RGBA `plane`, and an
/// encoded `image` — both `ImageRecord`s.
///
/// [`XRollover`]: crate::ui::x_rollover::XRollover
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XPiece {
    /// Placement origin + size words (axis order unresolved).
    pub geom: [i32; 4],
    /// `0x838 - rawmode` for raw kinds, else `2000`; `0x67` = empty image.
    pub kind: i32,
    // +0x14: implicit pad → plane aligns to +0x18.
    /// Raw RGBA plane (`dim` = plane `{ width, height }`).
    pub plane: ImageRecord,
    /// Encoded PNG image.
    pub image: ImageRecord,
}
