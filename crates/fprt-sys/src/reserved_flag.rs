//! The `ReservedFlag` config selector.

/// A reserved conductor-config field (`+0x48`): validated and stored, but never
/// read by the engine. Purpose unknown; both values are accepted (shipping
/// players send [`ReservedFlag::SECOND`]).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReservedFlag(pub u32);

impl ReservedFlag {
    /// `0x2dd661`.
    pub const FIRST: ReservedFlag = ReservedFlag(0x2dd661);
    /// `0x2dd662` — the value shipping players send.
    pub const SECOND: ReservedFlag = ReservedFlag(0x2dd662);
}
