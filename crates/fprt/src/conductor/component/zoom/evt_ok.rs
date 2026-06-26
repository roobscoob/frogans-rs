//! `ok` event (host → engine) — the zoom the user committed.

use fprt_sys::ui::zoom::event_ok::EventOk;
use fprt_sys::ui::zoom::EVT_OK;

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

const ZOOM_DEFAULT: i32 = 0x3e9;
const ZOOM_CUSTOM: i32 = 0x3ea;

/// The zoom the user committed.
pub enum ReportOk {
    /// Use the engine's default scaling factor.
    Default,
    /// A custom zoom in percent (the engine clamps to `50..=200`).
    Custom(i32),
}

impl sealed::Sealed for ReportOk {}

impl Report for ReportOk {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let (zoom_type, zoom_value) = match self {
            ReportOk::Default => (ZOOM_DEFAULT, 0),
            ReportOk::Custom(percent) => (ZOOM_CUSTOM, percent),
        };
        let payload = EventOk {
            event_id: EVT_OK,
            zoom_type,
            zoom_value,
        };
        // SAFETY: valid ctx; `payload` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().zoom_ok)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
