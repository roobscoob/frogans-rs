//! `update_steps_labels` command (engine → host) — the run-step combobox.

use fprt_sys::ui::inspector::update_steps_labels::UpdateStepsLabels as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_STEPS_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::{Pool, PooledString};

/// The inspector step-combobox entries plus the index to pre-select.
#[derive(Debug)]
pub struct UpdateStepsLabels {
    /// The target window.
    pub id: InspectorId,
    /// The step labels, in engine order.
    pub labels: Vec<PooledString>,
    /// Index into `labels` to pre-select.
    pub active_step: i32,
}

impl CommandPayload for UpdateStepsLabels {
    const ID: StatusName = CMD_UPDATE_STEPS_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_steps_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `labels` points at `count` entries written into `pool` by the pop.
        let labels = unsafe { pool.strings(raw.labels, raw.count) };
        UpdateStepsLabels {
            id: InspectorId(raw.reference),
            labels,
            active_step: raw.active_step,
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateStepsLabels(self)
    }
}
