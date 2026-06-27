//! `step_selected` event (host → engine) — the user picked a run step.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::inspector::ReportStepSelected;

impl sealed::Sealed for ReportStepSelected {}

impl Report for ReportStepSelected {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inspector_step_selected)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
