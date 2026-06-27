//! `open_directory` command (engine → host) — an enum, no pool.

use fprt_sys::ui::application::CMD_OPEN_DIRECTORY;
use fprt_sys::ui::application::open_dir_kind::OpenDirKind as RawOpenDirKind;
use fprt_sys::ui::application::open_directory::OpenDirectory as Raw;

/// Which known directory to reveal in the file manager.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenDirKind {
    /// `0xfa0` — default (no special handling).
    Default,
    /// `0xfa1` — the developers directory.
    Developers,
    /// An engine value outside the documented set.
    Other(u32),
}

impl OpenDirKind {
    /// Map the raw enum.
    pub fn from_raw(raw: RawOpenDirKind) -> Self {
        match raw {
            RawOpenDirKind::DEFAULT => OpenDirKind::Default,
            RawOpenDirKind::DEVELOPERS => OpenDirKind::Developers,
            other => OpenDirKind::Other(other.0),
        }
    }

    /// Map back to the raw enum.
    pub fn to_raw(self) -> RawOpenDirKind {
        match self {
            OpenDirKind::Default => RawOpenDirKind::DEFAULT,
            OpenDirKind::Developers => RawOpenDirKind::DEVELOPERS,
            OpenDirKind::Other(v) => RawOpenDirKind(v),
        }
    }
}

/// Reveal a known directory in the file manager. No path is carried — the host
/// knows the path for each [`OpenDirKind`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OpenDirectory {
    /// Which directory to reveal.
    pub kind: OpenDirKind,
}

impl OpenDirectory {
    /// Build one (no pool — enum payload).
    pub fn new(kind: OpenDirKind) -> Self {
        OpenDirectory { kind }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        OpenDirectory {
            kind: OpenDirKind::from_raw(raw.kind),
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            dir_id: CMD_OPEN_DIRECTORY,
            kind: self.kind.to_raw(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_every_variant() {
        for k in [
            OpenDirKind::Default,
            OpenDirKind::Developers,
            OpenDirKind::Other(0x9),
        ] {
            let p = OpenDirectory::new(k);
            assert_eq!(OpenDirectory::from_raw(p.to_raw()), p);
        }
    }
}
