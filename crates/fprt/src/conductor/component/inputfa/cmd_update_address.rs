//! `update_address` command (engine → host) — canonical address for the field.

use fprt_sys::ui::inputfa::update_address::UpdateAddress as Raw;
use fprt_sys::ui::inputfa::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The canonical Frogans Address text the engine wants shown in the input field
/// (e.g. after normalization).
#[derive(Debug)]
pub struct UpdateAddress {
    /// Frogans address text.
    pub address: Option<PooledString>,
}

impl CommandPayload for UpdateAddress {
    const ID: StatusName = CMD_UPDATE_ADDRESS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inputfa_update_address
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `address` was written into `pool` by the pop that produced both.
        let address = unsafe { pool.string(raw.address) };
        UpdateAddress { address }
    }

    fn into_command(self) -> Command {
        Command::InputfaUpdateAddress(self)
    }
}
