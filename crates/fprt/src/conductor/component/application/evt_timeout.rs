//! `timeout` event (host → engine) — a no-data marker (the host wake timer fired).

use crate::conductor::report::marker_event;

marker_event!(ReportTimeout, fprt_sys::ui::application::EVT_TIMEOUT, application_timeout);
