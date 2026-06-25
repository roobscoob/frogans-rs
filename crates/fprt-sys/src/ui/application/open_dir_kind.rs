//! The `OpenDirKind` selector (`command_open_directory`).

/// Which known directory to reveal. Only [`OpenDirKind::DEVELOPERS`] is exercised
/// by the shipping host.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OpenDirKind(pub u32);

impl OpenDirKind {
    /// `0xfa0` (4000) — default (no special handling).
    pub const DEFAULT: OpenDirKind = OpenDirKind(0xfa0);
    /// `0xfa1` (4001) — the developers directory (host reveals it).
    pub const DEVELOPERS: OpenDirKind = OpenDirKind(0xfa1);
}
