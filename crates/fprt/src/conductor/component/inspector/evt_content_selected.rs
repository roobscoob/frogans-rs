//! `content_selected` event (host → engine) — the user picked a content entry.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::inspector::ReportContentSelected;

impl sealed::Sealed for ReportContentSelected {}

impl Report for ReportContentSelected {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inspector_content_selected)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
