//! `update_layout` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::application::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::UpdateLayout;

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_layout
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::ApplicationUpdateLayout(UpdateLayout::from_raw(raw))
    }
}
