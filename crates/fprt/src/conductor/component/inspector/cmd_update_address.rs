//! `update_address` command (engine → host) — address shown in the inspector.

use fprt_sys::ui::inspector::update_address::UpdateAddress as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::UpdateAddress;

impl CommandPayload for UpdateAddress {
    const ID: StatusName = CMD_UPDATE_ADDRESS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_address
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InspectorUpdateAddress(UpdateAddress::from_raw(raw, pool))
    }
}
