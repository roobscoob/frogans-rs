//! `event_start` payload (`0x18`, IN).

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// Bootstrap event carrying the process locale to prime the initial render.
///
/// `locale` is `setlocale(LC_CTYPE, NULL)` (UTF-8) — the process locale, NOT a
/// Frogans address. The host owns the buffer; it need only outlive the call.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EventStart {
    /// Field 0 — must be `EVT_START` (`0x10ccca`).
    pub event_id: EventTag,
    // +0x04: 4 bytes implicit padding → locale aligns to +0x08.
    pub locale: Ustring,
}
