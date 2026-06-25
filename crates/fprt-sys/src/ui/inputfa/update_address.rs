//! `command_update_address` payload (`0x18`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The canonical Frogans Address text the engine hands the host to display in
/// the input field (e.g. after normalization).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateAddress {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → address aligns to +0x08.
    /// Frogans address text (`utf8` mempool-owned — free via `mempool_out`).
    pub address: Ustring,
}
