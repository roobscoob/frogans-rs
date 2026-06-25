//! `command_update_labels` payload (`0x58`) — title + four button strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The legal-information panel's five localized strings. Names are the engine
/// DWH-getter symbols (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// +0x04: command-reference word; keys an (unrecovered) finalization jumptable.
    pub variant_index: i32,
    pub title: Ustring,
    pub open_button: Ustring,
    pub select_button: Ustring,
    pub back_button: Ustring,
    pub close_button: Ustring,
}
