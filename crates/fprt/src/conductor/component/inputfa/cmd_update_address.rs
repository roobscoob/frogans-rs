//! `update_address` command (engine → host) — canonical address for the field.

use fprt_sys::ui::inputfa::update_address::UpdateAddress as Raw;
use fprt_sys::ui::inputfa::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inputfa::UpdateAddress;

impl CommandPayload for UpdateAddress {
    const ID: StatusName = CMD_UPDATE_ADDRESS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inputfa_update_address
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InputfaUpdateAddress(UpdateAddress::from_raw(raw, pool))
    }
}
