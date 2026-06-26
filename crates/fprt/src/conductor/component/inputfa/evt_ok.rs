//! `ok` event (host → engine) — the field text the user confirmed.

use fprt_sys::ui::inputfa::field_text::FieldText;
use fprt_sys::ui::inputfa::EVT_OK;

use crate::call::{invoke, ustring};
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The field text the user confirmed (borrowed for the call only).
pub struct ReportOk<'a> {
    text: &'a str,
}

impl<'a> ReportOk<'a> {
    /// The confirmed field text.
    pub fn new(text: &'a str) -> Self {
        ReportOk { text }
    }
}

impl sealed::Sealed for ReportOk<'_> {}

impl Report for ReportOk<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = FieldText {
            event_id: EVT_OK,
            text: ustring(self.text),
        };
        // SAFETY: valid ctx; `payload`/`self.text` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inputfa_ok)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
