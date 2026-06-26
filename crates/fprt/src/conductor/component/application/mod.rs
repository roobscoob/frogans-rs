//! `application` — top-level coordination.
//!
//! One file per call: `cmd_*` are engine→host commands (surfaced by
//! [`Command`](crate::Command)); `evt_*` are host→engine events (sent via
//! [`Conductor::report`](crate::Conductor::report)).

mod cmd_add_clipboard_image;
mod cmd_add_clipboard_text;
mod cmd_launch_way_out;
mod cmd_open_directory;
mod cmd_reinitialize_developers_directory;
mod cmd_stop;
mod cmd_update_directionality;
mod cmd_update_images;
mod cmd_update_layout;
mod cmd_update_zoom;
mod evt_change_layout;
mod evt_leaptofrogans;
mod evt_menu_access_unwanted;
mod evt_menu_access_wanted;
mod evt_quit;
mod evt_start;
mod evt_timeout;

pub use cmd_add_clipboard_image::AddClipboardImage;
pub use cmd_add_clipboard_text::AddClipboardText;
pub use cmd_launch_way_out::{LaunchWayOut, UriScheme};
pub use cmd_open_directory::{OpenDirKind, OpenDirectory};
pub use cmd_update_directionality::{Directionality, UpdateDirectionality};
pub use cmd_update_images::{Animation, UpdateImages};
pub use cmd_update_layout::UpdateLayout;
pub use cmd_update_zoom::UpdateZoom;
pub use evt_change_layout::{LayoutChange, ReportChangeLayout, SitehandlerLayout};
pub use evt_leaptofrogans::ReportLeaptofrogans;
pub use evt_menu_access_unwanted::ReportMenuAccessUnwanted;
pub use evt_menu_access_wanted::{MenuTarget, ReportMenuAccessWanted};
pub use evt_quit::ReportQuit;
pub use evt_start::ReportStart;
pub use evt_timeout::ReportTimeout;

// Marker types for the bare commands' dispatch (the public values are the unit
// variants `Command::ApplicationStop` / `Command::ApplicationReinitialize…`).
pub(crate) use cmd_reinitialize_developers_directory::ReinitializeDevelopersDirectory;
pub(crate) use cmd_stop::Stop;
