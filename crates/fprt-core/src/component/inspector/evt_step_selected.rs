//! `step_selected` event (host → engine) — the user picked a run step.

use fprt_sys::ui::inspector::EVT_STEP_SELECTED;
use fprt_sys::ui::inspector::step_selected::StepSelected as Raw;

use crate::component::inspector::InspectorId;

/// The user selected a step in one inspector window's run-step list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportStepSelected {
    /// The window the event came from.
    pub id: InspectorId,
    /// Index of the selected step.
    pub step_index: i32,
}

impl ReportStepSelected {
    /// Report that `step_index` was selected in window `id`.
    pub fn new(id: InspectorId, step_index: i32) -> Self {
        ReportStepSelected { id, step_index }
    }

    /// Decode an inbound payload.
    pub fn from_raw(raw: &Raw) -> Self {
        ReportStepSelected {
            id: InspectorId(raw.inspector_ref),
            step_index: raw.step_index,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_STEP_SELECTED,
            inspector_ref: self.id.0,
            step_index: self.step_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportStepSelected::new(InspectorId(3), 7);
        assert_eq!(ReportStepSelected::from_raw(&evt.to_raw()), evt);
    }
}
