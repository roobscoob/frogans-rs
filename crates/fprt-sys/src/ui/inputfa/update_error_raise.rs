//! `command_update_error_raise` payload (`0x18`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The localized error string the engine hands the host to display in the
/// dialog's inline error label (the user typed an invalid Frogans address).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateErrorRaise {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → error_msg aligns to +0x08.
    /// Inline error text (`utf8` mempool-owned — free via `mempool_out`).
    pub error_msg: Ustring,
}
