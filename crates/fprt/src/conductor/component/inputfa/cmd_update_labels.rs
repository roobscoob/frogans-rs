//! `update_labels` command (engine → host) — the five dialog strings.

use fprt_sys::ui::inputfa::labels::Labels as Raw;
use fprt_sys::ui::inputfa::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The five localized strings labelling the input-FA dialog chrome.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Window title.
    pub window_title: Option<PooledString>,
    /// Instruction text.
    pub instruction: Option<PooledString>,
    /// Input-field placeholder.
    pub input_placeholder: Option<PooledString>,
    /// OK-button label.
    pub ok_button_title: Option<PooledString>,
    /// Close-button label.
    pub close_button_title: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inputfa_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                window_title: pool.string(raw.window_title),
                instruction: pool.string(raw.instruction),
                input_placeholder: pool.string(raw.input_placeholder),
                ok_button_title: pool.string(raw.ok_button_title),
                close_button_title: pool.string(raw.close_button_title),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::InputfaUpdateLabels(self)
    }
}
