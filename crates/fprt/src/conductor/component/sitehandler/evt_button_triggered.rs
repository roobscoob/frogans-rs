//! `button_triggered` event (host → engine) — the user activated a site zone.

use fprt_sys::ui::sitehandler::button_triggered::ButtonTriggered;
use fprt_sys::ui::sitehandler::EVT_BUTTON_TRIGGERED;

use crate::call::{invoke, ustring};
use crate::conductor::component::sitehandler::SiteId;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The user activated an interactive zone in a rendered site. `entry_text` is the
/// typed text (ENTRY zones only, empty otherwise), borrowed for the call only.
pub struct ReportButtonTriggered<'a> {
    id: SiteId,
    button_index: i32,
    entry_text: &'a str,
}

impl<'a> ReportButtonTriggered<'a> {
    /// Report that zone `button_index` was activated in site `id`, carrying any
    /// `entry_text`.
    pub fn new(id: SiteId, button_index: i32, entry_text: &'a str) -> Self {
        ReportButtonTriggered {
            id,
            button_index,
            entry_text,
        }
    }
}

impl sealed::Sealed for ReportButtonTriggered<'_> {}

impl Report for ReportButtonTriggered<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = ButtonTriggered {
            event_id: EVT_BUTTON_TRIGGERED,
            site_id: self.id.0,
            button_index: self.button_index,
            entry_text: ustring(self.entry_text),
        };
        // SAFETY: valid ctx; `payload`/`self.entry_text` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().sitehandler_button_triggered)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
