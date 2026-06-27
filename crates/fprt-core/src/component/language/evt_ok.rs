//! `ok` event (host → engine) — the identifier of the chosen interface language
//! (a **borrowed** string).

use fprt_sys::ui::language::EVT_OK;
use fprt_sys::ui::language::event_ok::LanguageOk as Raw;

use crate::wire::{as_str, ustring};

/// The user confirmed a language; `lang_id` is its identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportOk<'a> {
    /// The chosen language's identifier, UTF-8.
    pub lang_id: &'a str,
}

impl<'a> ReportOk<'a> {
    /// Build one over a borrowed identifier.
    pub fn new(lang_id: &'a str) -> Self {
        ReportOk { lang_id }
    }

    /// Decode an inbound payload, borrowing its identifier for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.lang_id` is valid for the duration of the delivering call.
        ReportOk {
            lang_id: unsafe { as_str(raw.lang_id) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_OK,
            lang_id: ustring(self.lang_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportOk::new("en");
        assert_eq!(ReportOk::from_raw(&evt.to_raw()).lang_id, "en");
    }
}
