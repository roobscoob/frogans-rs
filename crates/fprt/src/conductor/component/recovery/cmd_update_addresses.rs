//! `update_addresses` command (engine → host) — the recoverable-address list.

use fprt_sys::ui::recovery::CMD_UPDATE_ADDRESSES;
use fprt_sys::ui::{AddressList as Raw, Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::recovery::UpdateAddresses;

impl CommandPayload for UpdateAddresses {
    const ID: StatusName = CMD_UPDATE_ADDRESSES;
    type Raw = Raw;
    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.recovery_update_addresses
    }
    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::RecoveryUpdateAddresses(UpdateAddresses::from_raw(raw, pool))
    }
}
