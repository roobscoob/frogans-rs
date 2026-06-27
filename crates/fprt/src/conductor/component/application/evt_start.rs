//! `start` event (host → engine) — client transport for the core codec.

use crate::call::invoke;
use crate::conductor::Conductor;
use crate::conductor::report::{Report, sealed};
use crate::error::EngineError;

pub use fprt_core::component::application::ReportStart;

impl sealed::Sealed for ReportStart<'_> {}

impl Report for ReportStart<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw` (and what it borrows) outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().application_start)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
