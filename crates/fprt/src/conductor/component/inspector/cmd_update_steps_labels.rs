//! `update_steps_labels` command (engine → host) — the run-step combobox.

use fprt_sys::ui::inspector::update_steps_labels::UpdateStepsLabels as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_STEPS_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::UpdateStepsLabels;

impl CommandPayload for UpdateStepsLabels {
    const ID: StatusName = CMD_UPDATE_STEPS_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_steps_labels
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InspectorUpdateStepsLabels(UpdateStepsLabels::from_raw(raw, pool))
    }
}
