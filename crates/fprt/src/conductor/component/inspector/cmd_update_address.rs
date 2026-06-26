//! `update_address` command (engine → host) — address shown in the inspector.

use fprt_sys::ui::inspector::update_address::UpdateAddress as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::{Pool, PooledString};

/// The Frogans address text shown in one inspector window's address field.
#[derive(Debug)]
pub struct UpdateAddress {
    /// The target window.
    pub id: InspectorId,
    /// Frogans address text.
    pub address: Option<PooledString>,
}

impl CommandPayload for UpdateAddress {
    const ID: StatusName = CMD_UPDATE_ADDRESS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_address
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `address` was written into `pool` by the pop that produced both.
        let address = unsafe { pool.string(raw.address) };
        UpdateAddress {
            id: InspectorId(raw.reference),
            address,
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateAddress(self)
    }
}
