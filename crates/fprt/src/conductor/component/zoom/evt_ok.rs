//! `ok` event (host → engine) — the zoom the user committed.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::zoom::ReportOk;

impl sealed::Sealed for ReportOk {}

impl Report for ReportOk {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().zoom_ok)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
