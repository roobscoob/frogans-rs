//! `ok` event (host → engine) — the zoom the user committed (an enum, no string).

use fprt_sys::ui::zoom::EVT_OK;
use fprt_sys::ui::zoom::event_ok::EventOk as Raw;

/// `0x3e9` — use the engine's default scaling factor.
const ZOOM_DEFAULT: i32 = 0x3e9;
/// `0x3ea` — use the carried percent value.
const ZOOM_CUSTOM: i32 = 0x3ea;

/// The zoom the user committed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReportOk {
    /// Use the engine's default scaling factor.
    Default,
    /// A custom zoom in percent (the engine clamps to `50..=200`).
    Custom(i32),
}

impl ReportOk {
    /// Decode an inbound payload.
    pub fn from_raw(raw: &Raw) -> Self {
        match raw.zoom_type {
            ZOOM_CUSTOM => ReportOk::Custom(raw.zoom_value),
            _ => ReportOk::Default,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        let (zoom_type, zoom_value) = match *self {
            ReportOk::Default => (ZOOM_DEFAULT, 0),
            ReportOk::Custom(percent) => (ZOOM_CUSTOM, percent),
        };
        Raw {
            event_id: EVT_OK,
            zoom_type,
            zoom_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_both_arms() {
        for r in [ReportOk::Default, ReportOk::Custom(150)] {
            assert_eq!(ReportOk::from_raw(&r.to_raw()), r);
        }
    }
}
