//! `event_button_triggered` payload (`0x18`, IN) — the only menu event.

use crate::ui::EventTag;
use crate::ustring::Ustring;

/// The user activated a menu button (clicked a command item, or submitted an
/// entry field — password / country code). The only way menu interaction enters
/// the engine.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ButtonTriggered {
    /// Field 0 — must be `EVT_BUTTON_TRIGGERED` (`0x10ccd4`).
    pub event_id: EventTag,
    /// The activated button's identifier — the engine's lookup key into the
    /// button list.
    pub button_index: i32,
    /// UTF-8 entry-field text (host-owned; empty if none).
    pub entry_text: Ustring,
}
