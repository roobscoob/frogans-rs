//! `content_selected` event (host → engine) — the user picked a content entry.

use fprt_sys::ui::inspector::EVT_CONTENT_SELECTED;
use fprt_sys::ui::inspector::content_selected::ContentSelected as Raw;

use crate::component::inspector::InspectorId;

/// The user selected a content entry in one inspector window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportContentSelected {
    /// The window the event came from.
    pub id: InspectorId,
    /// Index of the selected content entry.
    pub content_index: i32,
}

impl ReportContentSelected {
    /// Report that `content_index` was selected in window `id`.
    pub fn new(id: InspectorId, content_index: i32) -> Self {
        ReportContentSelected { id, content_index }
    }

    /// Decode an inbound payload.
    pub fn from_raw(raw: &Raw) -> Self {
        ReportContentSelected {
            id: InspectorId(raw.inspector_ref),
            content_index: raw.content_index,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_CONTENT_SELECTED,
            inspector_ref: self.id.0,
            content_index: self.content_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportContentSelected::new(InspectorId(4), 2);
        assert_eq!(ReportContentSelected::from_raw(&evt.to_raw()), evt);
    }
}
