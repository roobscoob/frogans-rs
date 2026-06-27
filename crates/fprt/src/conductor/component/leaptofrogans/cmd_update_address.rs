//! `update_address` command (engine → host) — the candidate address + compliance.

use fprt_sys::ui::leaptofrogans::update_address::UpdateAddress as Raw;
use fprt_sys::ui::leaptofrogans::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::leaptofrogans::UpdateAddress;

impl CommandPayload for UpdateAddress {
    const ID: StatusName = CMD_UPDATE_ADDRESS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.leaptofrogans_update_address
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::LeaptofrogansUpdateAddress(UpdateAddress::from_raw(raw, pool))
    }
}
