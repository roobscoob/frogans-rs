//! `update_labels` command (engine → host) — title + the button strings.

use fprt_sys::ui::leaptofrogans::labels::Labels as Raw;
use fprt_sys::ui::leaptofrogans::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The leap-to-Frogans dialog's strings. Two-armed: the normal view fills
/// `confirm`/`cancel`/`block`; the error view fills `close` and puts the error
/// text in `instruction`. The host detects the arm by whether `close_button` is
/// present.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Instruction (or, in the error arm, the error text).
    pub instruction: Option<PooledString>,
    /// Confirm-button label.
    pub confirm_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
    /// Block-button label.
    pub block_button: Option<PooledString>,
    /// Close-button label (present in the error arm).
    pub close_button: Option<PooledString>,
    /// Purge-button label (no macOS widget).
    pub purge_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.leaptofrogans_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                instruction: pool.string(raw.instruction),
                confirm_button: pool.string(raw.confirm_button),
                cancel_button: pool.string(raw.cancel_button),
                block_button: pool.string(raw.block_button),
                close_button: pool.string(raw.close_button),
                purge_button: pool.string(raw.purge_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::LeaptofrogansUpdateLabels(self)
    }
}
