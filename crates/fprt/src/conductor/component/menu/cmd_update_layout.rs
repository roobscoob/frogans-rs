//! `update_layout` command (engine → host) — menu geometry (host-discarded).

use fprt_sys::ui::menu::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::menu::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::menu::UpdateLayout;

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.menu_update_layout
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::MenuUpdateLayout(UpdateLayout::from_raw(raw))
    }
}
