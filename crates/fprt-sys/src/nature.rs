//! The `Nature` config selector.

/// The Frogans application's nature (conductor config `+0x58`); surfaced in the
/// manifest via `_adminsite_init`.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Nature(pub u32);

impl Nature {
    /// Public (`0x2de9e9`, engine-side `1`).
    pub const PUBLIC: Nature = Nature(0x2de9e9);
    /// Experimental (`0x2de9ea`, engine-side `2`).
    pub const EXPERIMENTAL: Nature = Nature(0x2de9ea);
}
