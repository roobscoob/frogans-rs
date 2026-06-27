//! `update_labels` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::favorites::CMD_UPDATE_LABELS;
use fprt_sys::ui::favorites::labels::Labels as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::favorites::UpdateLabels;

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.favorites_update_labels
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::FavoritesUpdateLabels(UpdateLabels::from_raw(raw, pool))
    }
}
