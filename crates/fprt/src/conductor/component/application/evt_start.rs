//! `start` event (host → engine).

use fprt_sys::ui::application as raw;

use crate::call::{invoke, ustring};
use crate::conductor::Conductor;
use crate::conductor::report::{Report, sealed};
use crate::error::EngineError;

/// Bootstrap event carrying the process locale (`setlocale(LC_CTYPE, NULL)`).
///
/// Borrows the locale for the call only — the engine reads it synchronously and
/// does not retain it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportStart<'a> {
    locale: &'a str,
}

impl<'a> ReportStart<'a> {
    /// `locale` is the process locale, UTF-8 (not a Frogans address).
    pub fn new(locale: &'a str) -> Self {
        ReportStart { locale }
    }
}

impl sealed::Sealed for ReportStart<'_> {}

impl Report for ReportStart<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = raw::event_start::EventStart {
            event_id: raw::EVT_START,
            locale: ustring(self.locale),
        };
        // SAFETY: valid ctx; `payload`/`self.locale` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().application_start)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
