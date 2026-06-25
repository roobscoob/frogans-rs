//! `command_add_clipboard_text` payload (`0x18`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// Text the host must place on the system clipboard.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AddClipboardText {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: 4 bytes implicit padding → text aligns to +0x08.
    /// Clipboard text (`utf8` is mempool-owned — free via the call's `mempool_out`).
    pub text: Ustring,
}
