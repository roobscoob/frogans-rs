//! `event_button_triggered` payload (`0x20`, IN).

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// The user activated an interactive zone in a rendered FSI. Carries the site
/// id, the zone index, and — only for an ENTRY zone — the typed text (host-owned).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ButtonTriggered {
    /// Field 0 — must be `EVT_BUTTON_TRIGGERED` (`0x10cced`).
    pub event_id: EventTag,
    /// `data_subset` id.
    pub site_id: i32,
    /// 0-based index into the pushed button list.
    pub button_index: i32,
    // +0x0c: implicit pad → entry_text aligns to +0x10.
    /// UTF-8 typed text (ENTRY zones only; empty otherwise).
    pub entry_text: Ustring,
}
