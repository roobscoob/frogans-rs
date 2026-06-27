//! `application` — top-level coordination payloads.
//!
//! `cmd_*` are engine→host commands (pooled where they carry strings/images);
//! `evt_*` are host→engine events (borrowed). Each type carries its own codec —
//! `new` / `from_raw` / `to_raw` — and knows nothing about the export table or
//! call sequencing (that's the client/server transport layer).
//!
mod cmd_add_clipboard_image;
mod cmd_add_clipboard_text;
mod cmd_launch_way_out;
mod cmd_open_directory;
mod cmd_update_directionality;
mod cmd_update_images;
mod cmd_update_layout;
mod cmd_update_zoom;
mod evt_change_layout;
mod evt_leaptofrogans;
mod evt_menu_access_wanted;
mod evt_start;

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
pub use evt_menu_access_wanted::{MenuTarget, ReportMenuAccessWanted};
pub use evt_start::ReportStart;
