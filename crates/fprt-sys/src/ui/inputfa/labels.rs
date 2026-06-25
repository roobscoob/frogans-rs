//! `command_update_labels` payload (`0x58`) — the five dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The five localized strings labelling the input-FA dialog chrome. Field names
/// are the host ivar / property symbols (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → window_title aligns to +0x08.
    pub window_title: Ustring,
    pub instruction: Ustring,
    pub input_placeholder: Ustring,
    pub ok_button_title: Ustring,
    pub close_button_title: Ustring,
}
