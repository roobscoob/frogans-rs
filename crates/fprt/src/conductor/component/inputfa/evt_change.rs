//! `change` event (host → engine) — the field text on every keystroke.

use fprt_sys::ui::inputfa::field_text::FieldText;
use fprt_sys::ui::inputfa::EVT_CHANGE;

use crate::call::{invoke, ustring};
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The input field's current text, reported as the user types (borrowed for the
/// call only).
pub struct ReportChange<'a> {
    text: &'a str,
}

impl<'a> ReportChange<'a> {
    /// The current field text.
    pub fn new(text: &'a str) -> Self {
        ReportChange { text }
    }
}

impl sealed::Sealed for ReportChange<'_> {}

impl Report for ReportChange<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = FieldText {
            event_id: EVT_CHANGE,
            text: ustring(self.text),
        };
        // SAFETY: valid ctx; `payload`/`self.text` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inputfa_change)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
