//! `start` event (host → engine) — a **borrowed** string.

use fprt_sys::ui::application::EVT_START;
use fprt_sys::ui::application::event_start::EventStart as Raw;

use crate::wire::{as_str, ustring};

/// Bootstrap event carrying the process locale (`setlocale(LC_CTYPE, NULL)` —
/// not a Frogans address).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportStart<'a> {
    /// The process locale, UTF-8.
    pub locale: &'a str,
}

impl<'a> ReportStart<'a> {
    /// Build one over a borrowed locale.
    pub fn new(locale: &'a str) -> Self {
        ReportStart { locale }
    }

    /// Decode an inbound payload, borrowing its locale for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.locale` is valid for the duration of the delivering call.
        ReportStart {
            locale: unsafe { as_str(raw.locale) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_START,
            locale: ustring(self.locale),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportStart::new("en_US.UTF-8");
        assert_eq!(ReportStart::from_raw(&evt.to_raw()).locale, "en_US.UTF-8");
    }
}
