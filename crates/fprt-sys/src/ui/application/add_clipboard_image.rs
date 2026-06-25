//! `command_add_clipboard_image` payload (`0x20`).

use crate::ui::{ImageRecord, StatusName};

/// Image the host must place on the system clipboard.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AddClipboardImage {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: 4 bytes implicit padding → image aligns to +0x08.
    /// Clipboard image (`buffer` is mempool-owned — free via the call's `mempool_out`).
    pub image: ImageRecord,
}
