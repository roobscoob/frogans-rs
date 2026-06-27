//! `update_addresses` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::favorites::CMD_UPDATE_ADDRESSES;
use fprt_sys::ui::{AddressList as Raw, Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::favorites::UpdateAddresses;

impl CommandPayload for UpdateAddresses {
    const ID: StatusName = CMD_UPDATE_ADDRESSES;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.favorites_update_addresses
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::FavoritesUpdateAddresses(UpdateAddresses::from_raw(raw, pool))
    }
}
