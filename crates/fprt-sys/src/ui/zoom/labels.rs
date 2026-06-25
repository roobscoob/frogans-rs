//! `command_update_labels` payload (`0x58`) — the five dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The five localized strings of the zoom-settings dialog. Field names are the
/// engine DWH-getter symbols (PROVEN). `restore_button` is engine-named but the
/// macOS host has no separate restore button (its binding is unresolved there).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → title aligns to +0x08.
    pub title: Ustring,
    pub default_button: Ustring,
    pub restore_button: Ustring,
    pub ok_button: Ustring,
    pub cancel_button: Ustring,
}
