//! `button_triggered` event (host → engine) — the user activated a site zone.

use fprt_sys::ui::sitehandler::EVT_BUTTON_TRIGGERED;
use fprt_sys::ui::sitehandler::button_triggered::ButtonTriggered as Raw;

use crate::component::sitehandler::SiteId;
use crate::wire::{as_str, ustring};

/// The user activated an interactive zone in a rendered site. `entry_text` is the
/// typed text (ENTRY zones only, empty otherwise), borrowed for the call only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportButtonTriggered<'a> {
    /// The site the zone belongs to.
    pub id: SiteId,
    /// 0-based index into the pushed button list.
    pub button_index: i32,
    /// Typed text (ENTRY zones only; empty otherwise).
    pub entry_text: &'a str,
}

impl<'a> ReportButtonTriggered<'a> {
    /// Report that zone `button_index` was activated in site `id`, carrying any
    /// `entry_text`.
    pub fn new(id: SiteId, button_index: i32, entry_text: &'a str) -> Self {
        ReportButtonTriggered {
            id,
            button_index,
            entry_text,
        }
    }

    /// Decode an inbound payload, borrowing its text for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.entry_text` is valid for the duration of the delivering call.
        ReportButtonTriggered {
            id: SiteId(raw.site_id),
            button_index: raw.button_index,
            entry_text: unsafe { as_str(raw.entry_text) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_BUTTON_TRIGGERED,
            site_id: self.id.0,
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
        let evt = ReportButtonTriggered::new(SiteId(4), 2, "frogans*alpha");
        let raw = evt.to_raw();
        let back = ReportButtonTriggered::from_raw(&raw);
        assert_eq!(back.id, SiteId(4));
        assert_eq!(back.button_index, 2);
        assert_eq!(back.entry_text, "frogans*alpha");
    }
}
