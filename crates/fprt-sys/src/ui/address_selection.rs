//! `AddressSelection` — the IN address-list event payload (`0x18`).

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// The entries the user selected, reported by a dialog's list events (host →
/// engine, IN). Same `0x18` layout as [`AddressList`], but field 0 is the
/// host-written event tag.
///
/// (A few components' list events carry user-facing *labels* rather than
/// addresses — same shape; the name reflects the common case.)
///
/// [`AddressList`]: crate::ui::address_list::AddressList
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AddressSelection {
    /// Field 0 — host-written event tag.
    pub event_id: EventTag,
    /// Reserved (`count` lives at +0x08, not +0x04).
    pub _rsv04: u32,
    /// Number of selected entries.
    pub count: u32,
    // +0x0c: implicit pad → items aligns to +0x10.
    /// Array of `count` selected entries (host-owned; stride `0x10`).
    pub items: *const Ustring,
}
