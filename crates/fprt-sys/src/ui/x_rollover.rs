//! `XRollover` — one interactive rollover region of a slide (`0x20`).

use crate::ui::x_piece::XPiece;

/// One clickable region of an [`XRepresentation`], carrying a per-state array of
/// image [`XPiece`]s.
///
/// [`XRepresentation`]: crate::ui::x_representation::XRepresentation
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct XRollover {
    /// Clickable rect words (origin + size; exact axis order unresolved).
    pub region: [i32; 4],
    /// Number of [`XPiece`]s.
    pub piece_count: i32,
    // +0x14: implicit pad → pieces aligns to +0x18.
    /// Mempool array of `piece_count` pieces (stride `0x48`).
    pub pieces: *mut XPiece,
}
