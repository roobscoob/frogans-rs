//! `update_labels` command (engine → host) — the inspector's full text set.

use fprt_sys::ui::inspector::labels::Labels as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::UpdateLabels;

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_labels
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InspectorUpdateLabels(UpdateLabels::from_raw(raw, pool))
    }
}
