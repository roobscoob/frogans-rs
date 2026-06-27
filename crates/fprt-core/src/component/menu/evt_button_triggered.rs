//! `button_triggered` event (host → engine) — the user activated a menu button.

use fprt_sys::ui::menu::EVT_BUTTON_TRIGGERED;
use fprt_sys::ui::menu::button_triggered::ButtonTriggered as Raw;

use crate::wire::{as_str, ustring};

/// The user activated a menu button — clicked a command item, or submitted an
/// entry field. `entry_text` is the field text (empty for non-entry buttons),
/// borrowed for the call only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportButtonTriggered<'a> {
    /// The activated button's identifier (the engine's lookup key).
    pub button_index: i32,
    /// Entry-field text (empty for non-entry buttons).
    pub entry_text: &'a str,
}

impl<'a> ReportButtonTriggered<'a> {
    /// Report that button `button_index` was activated, carrying any `entry_text`.
    pub fn new(button_index: i32, entry_text: &'a str) -> Self {
        ReportButtonTriggered {
            button_index,
            entry_text,
        }
    }

    /// Decode an inbound payload, borrowing its text for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.entry_text` is valid for the duration of the delivering call.
        ReportButtonTriggered {
            button_index: raw.button_index,
            entry_text: unsafe { as_str(raw.entry_text) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_BUTTON_TRIGGERED,
            button_index: self.button_index,
            entry_text: ustring(self.entry_text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportButtonTriggered::new(3, "café");
        let raw = evt.to_raw();
        let back = ReportButtonTriggered::from_raw(&raw);
        assert_eq!(back.button_index, 3);
        assert_eq!(back.entry_text, "café");
    }
}
