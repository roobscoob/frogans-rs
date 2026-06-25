//! `XRepresentation` — one rendered slide layer (`0x48`).

use crate::ui::x_rollover::XRollover;
use crate::ui::ImageRecord;

/// A rendered slide: the RGBA `image` plus its interactive rollover regions.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XRepresentation {
    /// Opaque dimensions/format handle (`_xrepresentation_get_raw_data`).
    pub raw_handle: u64,
    /// Rendered RGBA slide pixels.
    pub image: ImageRecord,
    /// Six geometry source words: origin (`[0..2]`) + size (`[2..4]`) + two whose
    /// role is unresolved (`[4..6]`).
    pub geom: [i32; 6],
    /// Number of [`XRollover`]s.
    pub rollover_count: i32,
    // +0x3c: implicit pad → rollovers aligns to +0x40.
    /// Mempool array of `rollover_count` rollovers (stride `0x20`).
    pub rollovers: *mut XRollover,
}
