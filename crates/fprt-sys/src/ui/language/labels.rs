//! `command_update_labels` payload (`0x58`) — the five dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The five localized strings of the language-selection dialog. Names are the
/// engine DWH-getter symbols (PROVEN). `cancel_button` drives the host's Close
/// button.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → title aligns to +0x08.
    pub title: Ustring,
    pub current: Ustring,
    pub select: Ustring,
    pub ok_button: Ustring,
    pub cancel_button: Ustring,
}
