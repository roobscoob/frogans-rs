//! `update_labels` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::blocked::CMD_UPDATE_LABELS;
use fprt_sys::ui::blocked::labels::Labels as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::blocked::UpdateLabels;

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.blocked_update_labels
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::BlockedUpdateLabels(UpdateLabels::from_raw(raw, pool))
    }
}
