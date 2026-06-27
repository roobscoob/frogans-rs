//! `update_labels` command (engine → host) — the dialog's localized strings.

use fprt_sys::ui::devtools::labels::Labels as Raw;
use fprt_sys::ui::devtools::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::devtools::UpdateLabels;

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;
    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.devtools_update_labels
    }
    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::DevtoolsUpdateLabels(UpdateLabels::from_raw(raw, pool))
    }
}
