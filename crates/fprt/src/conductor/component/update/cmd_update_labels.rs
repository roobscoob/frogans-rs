//! `update_labels` command (engine → host) — the dialog's localized strings.

use fprt_sys::ui::update::labels::Labels as Raw;
use fprt_sys::ui::update::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The four localized strings of the software-update dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Window title.
    pub window_title: Option<PooledString>,
    /// Instruction text (the notification-selected one).
    pub instruction_text: Option<PooledString>,
    /// Download-button title.
    pub download_button_title: Option<PooledString>,
    /// Cancel-button title.
    pub cancel_button_title: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.update_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                window_title: pool.string(raw.window_title),
                instruction_text: pool.string(raw.instruction_text),
                download_button_title: pool.string(raw.download_button_title),
                cancel_button_title: pool.string(raw.cancel_button_title),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::UpdateUpdateLabels(self)
    }
}
