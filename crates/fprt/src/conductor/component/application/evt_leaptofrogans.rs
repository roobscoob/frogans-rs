//! `leaptofrogans` event (host → engine) — open a Frogans address from the pad.

use fprt_sys::ui::application::event_leaptofrogans::EventLeaptofrogans;
use fprt_sys::ui::application::EVT_LEAPTOFROGANS;

use crate::call::{invoke, ustring};
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The host received a request to open a Frogans address (the pad is visible).
/// `address` is borrowed for the call only.
pub struct ReportLeaptofrogans<'a> {
    address: &'a str,
}

impl<'a> ReportLeaptofrogans<'a> {
    /// The Frogans address to open.
    pub fn new(address: &'a str) -> Self {
        ReportLeaptofrogans { address }
    }
}

impl sealed::Sealed for ReportLeaptofrogans<'_> {}

impl Report for ReportLeaptofrogans<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = EventLeaptofrogans {
            event_id: EVT_LEAPTOFROGANS,
            address: ustring(self.address),
        };
        // SAFETY: valid ctx; `payload`/`self.address` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().application_leaptofrogans)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
