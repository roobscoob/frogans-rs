//! `command_update_labels` payload (`0x68`) — the six dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The six localized UI strings of the recently-visited dialog. Field names are
/// the engine's DWH-getter symbols (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → title aligns to +0x08.
    pub title: Ustring,
    pub placeholder: Ustring,
    pub open_button: Ustring,
    pub delete_button: Ustring,
    pub delete_all_button: Ustring,
    pub cancel_button: Ustring,
}
