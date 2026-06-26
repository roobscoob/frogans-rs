//! `add_clipboard_text` command (engine → host).

use fprt_sys::ui::application::add_clipboard_text::AddClipboardText as Raw;
use fprt_sys::ui::application::CMD_ADD_CLIPBOARD_TEXT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// One pooled field — the clipboard text, a zero-copy view into the pool.
#[derive(Debug)]
pub struct AddClipboardText {
    /// The text, or `None` if the engine left it empty.
    pub text: Option<PooledString>,
}

impl CommandPayload for AddClipboardText {
    const ID: StatusName = CMD_ADD_CLIPBOARD_TEXT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_add_clipboard_text
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.text` was written into this `pool`'s mempool by the very
        // pop that produced both, so its bytes live as long as the pool.
        let text = unsafe { pool.string(raw.text) };
        AddClipboardText { text }
    }

    fn into_command(self) -> Command {
        Command::ApplicationAddClipboardText(self)
    }
}
