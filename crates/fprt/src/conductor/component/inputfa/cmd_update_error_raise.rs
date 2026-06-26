//! `update_error_raise` command (engine → host) — inline error text to display.

use fprt_sys::ui::inputfa::update_error_raise::UpdateErrorRaise as Raw;
use fprt_sys::ui::inputfa::CMD_UPDATE_ERROR_RAISE;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The localized error string the engine wants shown in the dialog's inline
/// error label (the user typed an invalid Frogans address).
#[derive(Debug)]
pub struct UpdateErrorRaise {
    /// Inline error text.
    pub error_msg: Option<PooledString>,
}

impl CommandPayload for UpdateErrorRaise {
    const ID: StatusName = CMD_UPDATE_ERROR_RAISE;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inputfa_update_error_raise
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `error_msg` was written into `pool` by the pop that produced both.
        let error_msg = unsafe { pool.string(raw.error_msg) };
        UpdateErrorRaise { error_msg }
    }

    fn into_command(self) -> Command {
        Command::InputfaUpdateErrorRaise(self)
    }
}
