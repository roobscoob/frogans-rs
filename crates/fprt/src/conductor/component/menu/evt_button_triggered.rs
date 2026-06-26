//! `button_triggered` event (host → engine) — the user activated a menu button.

use fprt_sys::ui::menu::button_triggered::ButtonTriggered;
use fprt_sys::ui::menu::EVT_BUTTON_TRIGGERED;

use crate::call::{invoke, ustring};
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The user activated a menu button — clicked a command item, or submitted an
/// entry field. `entry_text` is the field text (empty for non-entry buttons),
/// borrowed for the call only.
pub struct ReportButtonTriggered<'a> {
    button_index: i32,
    entry_text: &'a str,
}

impl<'a> ReportButtonTriggered<'a> {
    /// Report that button `button_index` was activated, carrying any `entry_text`.
    pub fn new(button_index: i32, entry_text: &'a str) -> Self {
        ReportButtonTriggered {
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
            button_index: self.button_index,
            entry_text: ustring(self.entry_text),
        };
        // SAFETY: valid ctx; `payload`/`self.entry_text` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().menu_button_triggered)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
