//! `change` event (host → engine) — the field text on every keystroke.

use fprt_sys::ui::inputfa::EVT_CHANGE;
use fprt_sys::ui::inputfa::field_text::FieldText as Raw;

use crate::wire::{as_str, ustring};

/// The input field's current text, reported as the user types (borrowed for the
/// call only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportChange<'a> {
    /// The current field text.
    pub text: &'a str,
}

impl<'a> ReportChange<'a> {
    /// Build one over a borrowed field text.
    pub fn new(text: &'a str) -> Self {
        ReportChange { text }
    }

    /// Decode an inbound payload, borrowing its text for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.text` is valid for the duration of the delivering call.
        ReportChange {
            text: unsafe { as_str(raw.text) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_CHANGE,
            text: ustring(self.text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportChange::new("frogans*α");
        assert_eq!(ReportChange::from_raw(&evt.to_raw()).text, "frogans*α");
    }
}
