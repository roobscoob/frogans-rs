//! `step_selected` event (host → engine) — the user picked a run step.

use fprt_sys::ui::inspector::step_selected::StepSelected;
use fprt_sys::ui::inspector::EVT_STEP_SELECTED;

use crate::call::invoke;
use crate::conductor::component::inspector::InspectorId;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The user selected a step in one inspector window's run-step list.
pub struct ReportStepSelected {
    id: InspectorId,
    step_index: i32,
}

impl ReportStepSelected {
    /// Report that `step_index` was selected in window `id`.
    pub fn new(id: InspectorId, step_index: i32) -> Self {
        ReportStepSelected { id, step_index }
    }
}

impl sealed::Sealed for ReportStepSelected {}

impl Report for ReportStepSelected {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = StepSelected {
            event_id: EVT_STEP_SELECTED,
            inspector_ref: self.id.0,
            step_index: self.step_index,
        };
        // SAFETY: valid ctx; `payload` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inspector_step_selected)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
