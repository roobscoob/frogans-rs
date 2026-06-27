//! `update_layout` command (engine → host) — the pad window's geometry.

use fprt_sys::ui::pad::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::pad::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::pad::UpdateLayout;

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.pad_update_layout
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::PadUpdateLayout(UpdateLayout::from_raw(raw))
    }
}
