//! `update_layout` command (engine → host) — an application layout scalar.

use fprt_sys::ui::application::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::application::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

/// An application-level layout value. The shipping host pops and discards it; the
/// scalar's meaning is unresolved, so it passes through verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateLayout {
    /// Application layout value (meaning unresolved — passthrough).
    pub layout_scalar: u32,
}

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_layout
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateLayout {
            layout_scalar: raw.layout_scalar,
        }
    }

    fn into_command(self) -> Command {
        Command::ApplicationUpdateLayout(self)
    }
}
