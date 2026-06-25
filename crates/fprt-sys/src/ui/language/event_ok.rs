//! `event_ok` payload (`0x18`, IN) — the chosen language identifier.

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// The identifier of the language the user confirmed. Host-owned; the engine
/// copies it during the call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LanguageOk {
    /// Field 0 — host-written event tag (`EVT_OK`).
    pub event_id: EventTag,
    // +0x04: implicit pad → lang_id aligns to +0x08.
    pub lang_id: Ustring,
}
