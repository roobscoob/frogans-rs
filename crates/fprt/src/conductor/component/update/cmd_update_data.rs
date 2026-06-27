//! `update_data` command (engine → host) — the dialog's two URIs.

use fprt_sys::ui::update::update_data::UpdateData as Raw;
use fprt_sys::ui::update::CMD_UPDATE_DATA;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::update::UpdateData;

impl CommandPayload for UpdateData {
    const ID: StatusName = CMD_UPDATE_DATA;
    type Raw = Raw;
    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.update_update_data
    }
    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::UpdateUpdateData(UpdateData::from_raw(raw, pool))
    }
}
