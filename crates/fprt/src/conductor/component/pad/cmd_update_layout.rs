//! `update_layout` command (engine → host) — the pad window's geometry.

use fprt_sys::ui::pad::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::pad::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::visual::ScreenRect;
use crate::pool::Pool;

/// The pad window's on-screen rectangle (`None` ⇒ no rect supplied). On the
/// macOS build the host discards this; modeled for completeness.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateLayout {
    /// Where to place the pad window.
    pub rect: Option<ScreenRect>,
}

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.pad_update_layout
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateLayout {
            rect: ScreenRect::option(raw.layout.present_flag, raw.layout.rect),
        }
    }

    fn into_command(self) -> Command {
        Command::PadUpdateLayout(self)
    }
}
