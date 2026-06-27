//! `button_triggered` event (host → engine) — the user activated a menu button.

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

pub use fprt_core::component::menu::ReportButtonTriggered;

impl sealed::Sealed for ReportButtonTriggered<'_> {}

impl Report for ReportButtonTriggered<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let raw = self.to_raw();
        // SAFETY: valid ctx; `raw`/`self.entry_text` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().menu_button_triggered)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
