//! `command_update_labels` payload (`0x58`) — the five dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The five localized UI strings of the blocked-addresses dialog. Names are the
/// host setter targets (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → title aligns to +0x08.
    pub title: Ustring,
    pub placeholder: Ustring,
    pub close_button: Ustring,
    pub remove_button: Ustring,
    pub remove_all_button: Ustring,
}
