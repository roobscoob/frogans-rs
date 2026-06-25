//! `AddressList` — the OUT address-list command payload (`0x18`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The variable-length list of Frogans addresses a dialog's `update_addresses`
/// command delivers to the host (engine → host, OUT). Shared across the list
/// dialogs (favorites, recentlyvisited, …).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AddressList {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// Reserved (`count` lives at +0x08, not +0x04).
    pub _rsv04: u32,
    /// Number of address entries.
    pub count: u32,
    // +0x0c: implicit pad → items aligns to +0x10.
    /// Mempool array of `count` addresses (stride `0x10`). NULL when empty.
    pub items: *const Ustring,
}
