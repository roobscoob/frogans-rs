//! `update_layout` command (engine → host) — menu geometry (host-discarded).

use fprt_sys::ui::menu::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::menu::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::visual::ScreenRect;
use crate::pool::Pool;

/// The menu's geometry (`None` ⇒ no rect supplied). The macOS host discards this
/// — the menu self-positions at the cursor — but it is modeled for completeness.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateLayout {
    /// Where the engine would place the menu.
    pub rect: Option<ScreenRect>,
}

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.menu_update_layout
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateLayout {
            rect: ScreenRect::option(raw.menu_layout.present_flag, raw.menu_layout.rect),
        }
    }

    fn into_command(self) -> Command {
        Command::MenuUpdateLayout(self)
    }
}
