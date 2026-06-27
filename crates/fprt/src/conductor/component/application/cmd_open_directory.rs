//! `open_directory` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_OPEN_DIRECTORY;
use fprt_sys::ui::application::open_directory::OpenDirectory as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::{OpenDirKind, OpenDirectory};

impl CommandPayload for OpenDirectory {
    const ID: StatusName = CMD_OPEN_DIRECTORY;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_open_directory
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::ApplicationOpenDirectory(OpenDirectory::from_raw(raw))
    }
}
