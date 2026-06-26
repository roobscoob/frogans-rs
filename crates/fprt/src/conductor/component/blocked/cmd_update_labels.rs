//! `update_labels` command (engine → host) — the dialog's localized strings.

use fprt_sys::ui::blocked::labels::Labels as Raw;
use fprt_sys::ui::blocked::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The five localized strings of the blocked-addresses dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Address-field placeholder.
    pub placeholder: Option<PooledString>,
    /// Close-button label.
    pub close_button: Option<PooledString>,
    /// Remove-button label.
    pub remove_button: Option<PooledString>,
    /// Remove-all-button label.
    pub remove_all_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.blocked_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                placeholder: pool.string(raw.placeholder),
                close_button: pool.string(raw.close_button),
                remove_button: pool.string(raw.remove_button),
                remove_all_button: pool.string(raw.remove_all_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::BlockedUpdateLabels(self)
    }
}
