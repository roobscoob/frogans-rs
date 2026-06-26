//! `change_autosync` event (host → engine) — the user toggled autosync.

use fprt_sys::ui::inspector::change_autosync::ChangeAutosync;
use fprt_sys::ui::inspector::EVT_CHANGE_AUTOSYNC;

use crate::call::invoke;
use crate::conductor::component::inspector::InspectorId;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The user toggled one inspector window's autosync mode.
pub struct ReportChangeAutosync {
    id: InspectorId,
    on: bool,
}

impl ReportChangeAutosync {
    /// Report the new autosync state (`on`) for window `id`.
    pub fn new(id: InspectorId, on: bool) -> Self {
        ReportChangeAutosync { id, on }
    }
}

impl sealed::Sealed for ReportChangeAutosync {}

impl Report for ReportChangeAutosync {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = ChangeAutosync {
            event_id: EVT_CHANGE_AUTOSYNC,
            inspector_ref: self.id.0,
            // `0xbb9` = ON (mode 1), `0xbba` = OFF (mode 2).
            autosync_mode_enum: if self.on { 0xbb9 } else { 0xbba },
        };
        // SAFETY: valid ctx; `payload` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inspector_change_autosync)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
