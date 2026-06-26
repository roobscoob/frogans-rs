//! `update_labels` command (engine → host) — the five dialog strings.

use fprt_sys::ui::language::labels::Labels as Raw;
use fprt_sys::ui::language::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The five localized strings of the language-selection dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// "Current language" label.
    pub current: Option<PooledString>,
    /// "Select" prompt label.
    pub select: Option<PooledString>,
    /// OK-button label.
    pub ok_button: Option<PooledString>,
    /// Cancel/Close-button label.
    pub cancel_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.language_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                current: pool.string(raw.current),
                select: pool.string(raw.select),
                ok_button: pool.string(raw.ok_button),
                cancel_button: pool.string(raw.cancel_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::LanguageUpdateLabels(self)
    }
}
