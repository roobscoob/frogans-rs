//! `ok` event (host → engine) — the identifier of the chosen interface language.

use fprt_sys::ui::language::event_ok::LanguageOk;
use fprt_sys::ui::language::EVT_OK;

use crate::call::{invoke, ustring};
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The user confirmed a language; `lang_id` is its identifier (borrowed for the
/// call only).
pub struct ReportOk<'a> {
    lang_id: &'a str,
}

impl<'a> ReportOk<'a> {
    /// The chosen language's identifier.
    pub fn new(lang_id: &'a str) -> Self {
        ReportOk { lang_id }
    }
}

impl sealed::Sealed for ReportOk<'_> {}

impl Report for ReportOk<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = LanguageOk {
            event_id: EVT_OK,
            lang_id: ustring(self.lang_id),
        };
        // SAFETY: valid ctx; `payload`/`self.lang_id` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().language_ok)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
