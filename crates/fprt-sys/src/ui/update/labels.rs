//! `command_update_labels` payload (`0x48`) — the four dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The four localized strings of the software-update dialog. Only one
/// instruction string is carried (the notification-selected one). Names are the
/// host `UIUpdateController` ivars (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → window_title aligns to +0x08.
    pub window_title: Ustring,
    pub instruction_text: Ustring,
    pub download_button_title: Ustring,
    pub cancel_button_title: Ustring,
}
