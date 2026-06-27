//! `update_error_raise` command (engine → host) — inline error text to display.

use fprt_sys::ui::inputfa::update_error_raise::UpdateErrorRaise as Raw;
use fprt_sys::ui::inputfa::CMD_UPDATE_ERROR_RAISE;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inputfa::UpdateErrorRaise;

impl CommandPayload for UpdateErrorRaise {
    const ID: StatusName = CMD_UPDATE_ERROR_RAISE;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inputfa_update_error_raise
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InputfaUpdateErrorRaise(UpdateErrorRaise::from_raw(raw, pool))
    }
}
