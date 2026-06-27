//! `add_clipboard_text` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_ADD_CLIPBOARD_TEXT;
use fprt_sys::ui::application::add_clipboard_text::AddClipboardText as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::AddClipboardText;

impl CommandPayload for AddClipboardText {
    const ID: StatusName = CMD_ADD_CLIPBOARD_TEXT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_add_clipboard_text
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::ApplicationAddClipboardText(AddClipboardText::from_raw(raw, pool))
    }
}
