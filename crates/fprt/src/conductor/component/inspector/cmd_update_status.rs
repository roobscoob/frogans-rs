//! `update_status` command (engine → host) — run outcome + data-available flag.

use fprt_sys::ui::inspector::update_status::UpdateStatus as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_STATUS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::{RunStatus, UpdateStatus};

impl CommandPayload for UpdateStatus {
    const ID: StatusName = CMD_UPDATE_STATUS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_status
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::InspectorUpdateStatus(UpdateStatus::from_raw(raw))
    }
}
