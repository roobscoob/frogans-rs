//! `ok` event (host → engine) — the identifier of the chosen interface language.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::language::ReportOk;

impl sealed::Sealed for ReportOk<'_> {}

impl Report for ReportOk<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw`/its borrowed identifier outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().language_ok)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
