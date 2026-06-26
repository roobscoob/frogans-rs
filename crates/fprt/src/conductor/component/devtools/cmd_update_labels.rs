//! `update_labels` command (engine → host) — the dialog's localized strings.

use fprt_sys::ui::devtools::labels::Labels as Raw;
use fprt_sys::ui::devtools::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The four localized strings of the developers-directory dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Address-field placeholder.
    pub placeholder: Option<PooledString>,
    /// Inspect-button label.
    pub inspect_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.devtools_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                placeholder: pool.string(raw.placeholder),
                inspect_button: pool.string(raw.inspect_button),
                cancel_button: pool.string(raw.cancel_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::DevtoolsUpdateLabels(self)
    }
}
