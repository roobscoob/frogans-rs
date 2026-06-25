//! The `change` / `ok` event payload (`0x18`, IN) — both carry the field text.

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// The input field's text, reported on `change` (every keystroke) and on `ok`
/// (the user confirmed). Host-owned; the engine copies it during the call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FieldText {
    /// Field 0 — host-written event tag (`EVT_CHANGE` or `EVT_OK`).
    pub event_id: EventTag,
    // +0x04: implicit pad → text aligns to +0x08.
    /// UTF-8 field text.
    pub text: Ustring,
}
