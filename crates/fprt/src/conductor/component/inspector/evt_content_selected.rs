//! `content_selected` event (host → engine) — the user picked a content entry.

use fprt_sys::ui::inspector::content_selected::ContentSelected;
use fprt_sys::ui::inspector::EVT_CONTENT_SELECTED;

use crate::call::invoke;
use crate::conductor::component::inspector::InspectorId;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// The user selected a content entry in one inspector window.
pub struct ReportContentSelected {
    id: InspectorId,
    content_index: i32,
}

impl ReportContentSelected {
    /// Report that `content_index` was selected in window `id`.
    pub fn new(id: InspectorId, content_index: i32) -> Self {
        ReportContentSelected { id, content_index }
    }
}

impl sealed::Sealed for ReportContentSelected {}

impl Report for ReportContentSelected {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let payload = ContentSelected {
            event_id: EVT_CONTENT_SELECTED,
            inspector_ref: self.id.0,
            content_index: self.content_index,
        };
        // SAFETY: valid ctx; `payload` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().inspector_content_selected)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
