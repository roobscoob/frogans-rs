//! `add_clipboard_image` command (engine → host) — image to put on the clipboard.

use fprt_sys::ui::application::add_clipboard_image::AddClipboardImage as Raw;
use fprt_sys::ui::application::CMD_ADD_CLIPBOARD_IMAGE;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledImage};

/// An image the host must place on the system clipboard.
#[derive(Debug)]
pub struct AddClipboardImage {
    /// The clipboard image.
    pub image: Option<PooledImage>,
}

impl CommandPayload for AddClipboardImage {
    const ID: StatusName = CMD_ADD_CLIPBOARD_IMAGE;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_add_clipboard_image
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `image` was written into `pool` by the pop that produced both.
        let image = unsafe { pool.image(raw.image) };
        AddClipboardImage { image }
    }

    fn into_command(self) -> Command {
        Command::ApplicationAddClipboardImage(self)
    }
}
