//! `update_labels` command (engine → host) — title + the button strings.

use fprt_sys::ui::legalinformation::labels::Labels as Raw;
use fprt_sys::ui::legalinformation::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The five localized strings of the legal-information panel.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Panel title.
    pub title: Option<PooledString>,
    /// Open-button label.
    pub open_button: Option<PooledString>,
    /// Select-button label.
    pub select_button: Option<PooledString>,
    /// Back-button label.
    pub back_button: Option<PooledString>,
    /// Close-button label.
    pub close_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.legalinformation_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                open_button: pool.string(raw.open_button),
                select_button: pool.string(raw.select_button),
                back_button: pool.string(raw.back_button),
                close_button: pool.string(raw.close_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::LegalinformationUpdateLabels(self)
    }
}
