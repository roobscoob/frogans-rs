//! `update_sync` command (engine → host) — autosync state + synchronize-enable.

use fprt_sys::ui::inspector::update_sync::UpdateSync as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_SYNC;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::UpdateSync;

impl CommandPayload for UpdateSync {
    const ID: StatusName = CMD_UPDATE_SYNC;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_sync
    }

    fn decode(raw: Raw, _pool: &Pool) -> Command {
        Command::InspectorUpdateSync(UpdateSync::from_raw(raw))
    }
}
