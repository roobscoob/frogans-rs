//! `update_address` command (engine → host) — the candidate address + compliance.

use fprt_sys::ui::leaptofrogans::update_address::UpdateAddress as Raw;
use fprt_sys::ui::leaptofrogans::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The Frogans address being evaluated, plus whether it's compliant (drives
/// which buttons the host shows).
#[derive(Debug)]
pub struct UpdateAddress {
    /// The candidate Frogans address.
    pub address: Option<PooledString>,
    /// Whether the address is compliant.
    pub compliant: bool,
}

impl CommandPayload for UpdateAddress {
    const ID: StatusName = CMD_UPDATE_ADDRESS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.leaptofrogans_update_address
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.address` was written into `pool` by the pop that produced both.
        let address = unsafe { pool.string(raw.address) };
        UpdateAddress {
            address,
            compliant: raw.compliant_address == 1,
        }
    }

    fn into_command(self) -> Command {
        Command::LeaptofrogansUpdateAddress(self)
    }
}
