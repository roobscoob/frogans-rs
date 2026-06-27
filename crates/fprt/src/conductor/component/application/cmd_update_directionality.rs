//! `update_directionality` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_UPDATE_DIRECTIONALITY;
use fprt_sys::ui::application::update_directionality::UpdateDirectionality as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::{Directionality, UpdateDirectionality};

impl CommandPayload for UpdateDirectionality {
    const ID: StatusName = CMD_UPDATE_DIRECTIONALITY;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_directionality
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::ApplicationUpdateDirectionality(UpdateDirectionality::from_raw(raw))
    }
}
