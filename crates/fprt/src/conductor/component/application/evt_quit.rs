//! `quit` event (host → engine) — a no-data marker.

use crate::conductor::report::marker_event;

marker_event!(ReportQuit, fprt_sys::ui::application::EVT_QUIT, application_quit);
