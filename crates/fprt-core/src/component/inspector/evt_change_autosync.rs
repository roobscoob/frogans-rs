//! `change_autosync` event (host → engine) — the user toggled autosync.

use fprt_sys::ui::inspector::EVT_CHANGE_AUTOSYNC;
use fprt_sys::ui::inspector::change_autosync::ChangeAutosync as Raw;

use crate::component::inspector::InspectorId;

/// The user toggled one inspector window's autosync mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportChangeAutosync {
    /// The window the event came from.
    pub id: InspectorId,
    /// New autosync state: `true` = ON (`0xbb9`), `false` = OFF (`0xbba`).
    pub on: bool,
}

impl ReportChangeAutosync {
    /// Report the new autosync state (`on`) for window `id`.
    pub fn new(id: InspectorId, on: bool) -> Self {
        ReportChangeAutosync { id, on }
    }

    /// Decode an inbound payload.
    pub fn from_raw(raw: &Raw) -> Self {
        ReportChangeAutosync {
            id: InspectorId(raw.inspector_ref),
            on: raw.autosync_mode_enum == 0xbb9,
        }
    }

    /// Encode into the raw payload (`0xbb9` = ON, `0xbba` = OFF).
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_CHANGE_AUTOSYNC,
            inspector_ref: self.id.0,
            autosync_mode_enum: if self.on { 0xbb9 } else { 0xbba },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_both_states() {
        for on in [true, false] {
            let evt = ReportChangeAutosync::new(InspectorId(8), on);
            assert_eq!(ReportChangeAutosync::from_raw(&evt.to_raw()), evt);
        }
    }
}
