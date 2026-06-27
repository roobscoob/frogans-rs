//! `change_layout` event (host → engine) — client transport for the core codec.

use crate::call::invoke;
use crate::conductor::Conductor;
use crate::conductor::report::{Report, sealed};
use crate::error::EngineError;

pub use fprt_core::component::application::{LayoutChange, ReportChangeLayout, SitehandlerLayout};

impl sealed::Sealed for ReportChangeLayout {}

impl Report for ReportChangeLayout {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let scratch = crate::pool::OwnedPool::new();
        let raw = self.to_raw(&scratch);
        // SAFETY: valid ctx; `raw` + `scratch` (which backs its array) outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().application_change_layout)(ctx, &raw, s, e, p)
        })
        .map(|_| ())
    }
}
