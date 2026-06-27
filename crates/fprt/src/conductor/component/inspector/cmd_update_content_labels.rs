//! `update_content_labels` command (engine → host) — the content selector.

use fprt_sys::ui::inspector::update_content_labels::UpdateContentLabels as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_CONTENT_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::UpdateContentLabels;

impl CommandPayload for UpdateContentLabels {
    const ID: StatusName = CMD_UPDATE_CONTENT_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_content_labels
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InspectorUpdateContentLabels(UpdateContentLabels::from_raw(raw, pool))
    }
}
