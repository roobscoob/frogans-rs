//! `change_autosync` event (host → engine) — the user toggled autosync.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::inspector::ReportChangeAutosync;

impl sealed::Sealed for ReportChangeAutosync {}

impl Report for ReportChangeAutosync {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inspector_change_autosync)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
