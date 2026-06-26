//! `update_addresses` command (engine → host) — the recoverable-address list.

use fprt_sys::ui::recovery::CMD_UPDATE_ADDRESSES;
use fprt_sys::ui::{AddressList as Raw, Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The list of recoverable Frogans addresses.
#[derive(Debug)]
pub struct UpdateAddresses {
    /// The addresses (null/empty dropped).
    pub addresses: Vec<PooledString>,
}

impl CommandPayload for UpdateAddresses {
    const ID: StatusName = CMD_UPDATE_ADDRESSES;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.recovery_update_addresses
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.items` is `raw.count` `Ustring`s the pop wrote into `pool`.
        let addresses = unsafe { pool.strings(raw.items, raw.count) };
        UpdateAddresses { addresses }
    }

    fn into_command(self) -> Command {
        Command::RecoveryUpdateAddresses(self)
    }
}
