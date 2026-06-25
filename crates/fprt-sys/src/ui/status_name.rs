//! The `StatusName` command type-tag (field 0 of every command payload).

/// The `0x2195xx` command-class id the engine stamps into a command payload's
/// field 0, looked up from the 124-entry status-name table.
///
/// The engine pre-seeds [`StatusName::NONE`] before a pop and overwrites it on
/// success (or [`StatusName::FALLBACK`] for an out-of-range index). Per-component
/// command ids live with their component.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StatusName(pub u32);

impl StatusName {
    /// "no command" — the engine's pre-pop seed.
    pub const NONE: StatusName = StatusName(0x2195a8);
    /// Default / out-of-range fallback.
    pub const FALLBACK: StatusName = StatusName(0x2195a9);
}
