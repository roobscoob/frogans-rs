//! `update_content_labels` command (engine → host) — the content selector.

use fprt_sys::ui::inspector::update_content_labels::UpdateContentLabels as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_CONTENT_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::{Pool, PooledString};

/// The inspector content-selector entries plus the active index.
#[derive(Debug)]
pub struct UpdateContentLabels {
    /// The target window.
    pub id: InspectorId,
    /// The content labels, in engine order.
    pub labels: Vec<PooledString>,
    /// Index into `labels` that is selected/active.
    pub content_active: i32,
}

impl CommandPayload for UpdateContentLabels {
    const ID: StatusName = CMD_UPDATE_CONTENT_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_content_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `labels` points at `count` entries written into `pool` by the pop.
        let labels = unsafe { pool.strings(raw.labels, raw.count) };
        UpdateContentLabels {
            id: InspectorId(raw.reference),
            labels,
            content_active: raw.content_active,
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateContentLabels(self)
    }
}
