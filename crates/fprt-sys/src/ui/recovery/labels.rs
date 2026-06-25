//! `command_update_labels` payload (`0x48`) — the four dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The four localized UI strings of the recovery dialog. Names are the engine
/// DWH-getter symbols (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → title aligns to +0x08.
    pub title: Ustring,
    pub placeholder: Ustring,
    pub open_button: Ustring,
    pub cancel_button: Ustring,
}
