//! `update_labels` command (engine → host) — the dialog's localized strings.

use fprt_sys::ui::zoom::labels::Labels as Raw;
use fprt_sys::ui::zoom::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The five localized strings of the zoom-settings dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Default-button label.
    pub default_button: Option<PooledString>,
    /// Restore-button label (no separate button on the macOS host).
    pub restore_button: Option<PooledString>,
    /// OK-button label.
    pub ok_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.zoom_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                default_button: pool.string(raw.default_button),
                restore_button: pool.string(raw.restore_button),
                ok_button: pool.string(raw.ok_button),
                cancel_button: pool.string(raw.cancel_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::ZoomUpdateLabels(self)
    }
}
