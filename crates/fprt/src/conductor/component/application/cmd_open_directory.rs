//! `open_directory` command (engine → host) — reveal a known directory.

use fprt_sys::ui::application::open_dir_kind::OpenDirKind as RawOpenDirKind;
use fprt_sys::ui::application::open_directory::OpenDirectory as Raw;
use fprt_sys::ui::application::CMD_OPEN_DIRECTORY;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

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
    fn from_raw(raw: RawOpenDirKind) -> Self {
        match raw {
            RawOpenDirKind::DEFAULT => OpenDirKind::Default,
            RawOpenDirKind::DEVELOPERS => OpenDirKind::Developers,
            _ => OpenDirKind::Other(raw.0),
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

impl CommandPayload for OpenDirectory {
    const ID: StatusName = CMD_OPEN_DIRECTORY;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_open_directory
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        OpenDirectory {
            kind: OpenDirKind::from_raw(raw.kind),
        }
    }

    fn into_command(self) -> Command {
        Command::ApplicationOpenDirectory(self)
    }
}
