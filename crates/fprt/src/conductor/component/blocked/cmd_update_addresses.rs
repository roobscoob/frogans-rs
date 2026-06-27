//! `update_addresses` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::blocked::CMD_UPDATE_ADDRESSES;
use fprt_sys::ui::{AddressList as Raw, Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::blocked::UpdateAddresses;

impl CommandPayload for UpdateAddresses {
    const ID: StatusName = CMD_UPDATE_ADDRESSES;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.blocked_update_addresses
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::BlockedUpdateAddresses(UpdateAddresses::from_raw(raw, pool))
    }
}
