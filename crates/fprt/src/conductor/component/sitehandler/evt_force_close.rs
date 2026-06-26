//! `force_close` event (host → engine) — force a site closed. Dormant on macOS.

use fprt_sys::ui::sitehandler::force_close::ForceClose;
use fprt_sys::ui::sitehandler::EVT_FORCE_CLOSE;

use crate::call::invoke;
use crate::conductor::component::sitehandler::SiteId;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// Force a Frogans Site closed. Carries only the site id. (Dormant on the macOS
/// host — no sender is wired there — but modeled for completeness.)
pub struct ReportForceClose {
    id: SiteId,
}

impl ReportForceClose {
    /// Force-close site `id`.
    pub fn new(id: SiteId) -> Self {
        ReportForceClose { id }
    }
}

impl sealed::Sealed for ReportForceClose {}

impl Report for ReportForceClose {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = ForceClose {
            event_id: EVT_FORCE_CLOSE,
            site_id: self.id.0,
        };
        // SAFETY: valid ctx; `payload` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().sitehandler_force_close)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
