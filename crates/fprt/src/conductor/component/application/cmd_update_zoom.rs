//! `update_zoom` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_UPDATE_ZOOM;
use fprt_sys::ui::application::update_zoom::UpdateZoom as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::UpdateZoom;

impl CommandPayload for UpdateZoom {
    const ID: StatusName = CMD_UPDATE_ZOOM;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_zoom
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::ApplicationUpdateZoom(UpdateZoom::from_raw(raw))
    }
}
