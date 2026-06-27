//! `update_visual` command (engine → host) — the rendered menu + entry buttons.

use fprt_sys::ui::menu::update_visual::UpdateVisual as Raw;
use fprt_sys::ui::menu::CMD_UPDATE_VISUAL;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::menu::{MenuVariant, UpdateVisual};

impl CommandPayload for UpdateVisual {
    const ID: StatusName = CMD_UPDATE_VISUAL;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.menu_update_visual
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::MenuUpdateVisual(UpdateVisual::from_raw(raw, pool))
    }
}
