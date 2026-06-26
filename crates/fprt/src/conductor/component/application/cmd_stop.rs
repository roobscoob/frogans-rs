//! `stop` command (engine → host) — a no-payload marker.

use crate::conductor::command::marker_command;

marker_command!(
    Stop,
    fprt_sys::ui::application::CMD_STOP,
    application_stop,
    ApplicationStop
);
