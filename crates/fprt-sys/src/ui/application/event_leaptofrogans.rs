//! `event_leaptofrogans` payload (`0x18`, IN).

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// The host received a request to open a Frogans address (the pad is visible).
/// The host owns `address`; it need only outlive the call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EventLeaptofrogans {
    /// Field 0 — must be `EVT_LEAPTOFROGANS` (`0x10ccd1`).
    pub event_id: EventTag,
    // +0x04: 4 bytes implicit padding → address aligns to +0x08.
    /// UTF-8 Frogans address.
    pub address: Ustring,
}
