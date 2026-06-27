//! `ok` event (host → engine) — the field text the user confirmed.

use fprt_sys::ui::inputfa::EVT_OK;
use fprt_sys::ui::inputfa::field_text::FieldText as Raw;

use crate::wire::{as_str, ustring};

/// The field text the user confirmed (borrowed for the call only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportOk<'a> {
    /// The confirmed field text.
    pub text: &'a str,
}

impl<'a> ReportOk<'a> {
    /// Build one over a borrowed field text.
    pub fn new(text: &'a str) -> Self {
        ReportOk { text }
    }

    /// Decode an inbound payload, borrowing its text for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.text` is valid for the duration of the delivering call.
        ReportOk {
            text: unsafe { as_str(raw.text) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_OK,
            text: ustring(self.text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportOk::new("frogans*confirmed");
        assert_eq!(ReportOk::from_raw(&evt.to_raw()).text, "frogans*confirmed");
    }
}
