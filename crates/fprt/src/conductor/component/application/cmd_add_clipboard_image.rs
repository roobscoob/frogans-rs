//! `add_clipboard_image` command (engine → host) — client transport for the core codec.

use fprt_sys::Fprt;
use fprt_sys::ui::application::CMD_ADD_CLIPBOARD_IMAGE;
use fprt_sys::ui::application::add_clipboard_image::AddClipboardImage as Raw;
use fprt_sys::ui::{Pop, StatusName};

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::application::AddClipboardImage;

impl CommandPayload for AddClipboardImage {
    const ID: StatusName = CMD_ADD_CLIPBOARD_IMAGE;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_add_clipboard_image
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::ApplicationAddClipboardImage(AddClipboardImage::from_raw(raw, pool))
    }
}
