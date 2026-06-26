//! `menu_access_unwanted` event (host → engine) — a no-data marker (dismiss menu).

use crate::conductor::report::marker_event;

marker_event!(
    ReportMenuAccessUnwanted,
    fprt_sys::ui::application::EVT_MENU_ACCESS_UNWANTED,
    application_menu_access_unwanted
);
