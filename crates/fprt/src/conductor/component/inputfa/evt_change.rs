//! `change` event (host → engine) — the field text on every keystroke.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::inputfa::ReportChange;

impl sealed::Sealed for ReportChange<'_> {}

impl Report for ReportChange<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw`/`self.text` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inputfa_change)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
