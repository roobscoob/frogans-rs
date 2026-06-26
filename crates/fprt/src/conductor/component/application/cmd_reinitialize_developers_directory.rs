//! `reinitialize_developers_directory` command (engine → host) — a no-payload marker.

use crate::conductor::command::marker_command;

marker_command!(
    ReinitializeDevelopersDirectory,
    fprt_sys::ui::application::CMD_REINIT_DEV_DIR,
    application_reinitialize_developers_directory,
    ApplicationReinitializeDevelopersDirectory
);
