//! application — top-level coordination (`fprt_ui_application_*`).
//!
//! 10 commands (engine → host, [`Pop`]) + 7 events (host → engine, [`Report`]).
//! All 17 share the uniform 5-arg envelope; field layouts live in the per-payload
//! modules. Command statuses are `0x17e9xxxx`, event statuses `0x17e8xxxx`.

pub mod add_clipboard_image;
pub mod add_clipboard_text;
pub mod directionality;
pub mod event_change_layout;
pub mod event_leaptofrogans;
pub mod event_menu_access_wanted;
pub mod event_start;
pub mod launch_way_out;
pub mod menu_variant;
pub mod open_dir_kind;
pub mod open_directory;
pub mod update_directionality;
pub mod update_images;
pub mod update_layout;
pub mod update_zoom;
pub mod uri_scheme;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags (engine stamps payload field 0; host maps the result) ---
pub const CMD_UPDATE_IMAGES: StatusName = StatusName(0x2195aa);
pub const CMD_UPDATE_ZOOM: StatusName = StatusName(0x2195ab);
pub const CMD_UPDATE_LAYOUT: StatusName = StatusName(0x2195ac);
pub const CMD_UPDATE_DIRECTIONALITY: StatusName = StatusName(0x2195ad);
pub const CMD_ADD_CLIPBOARD_TEXT: StatusName = StatusName(0x2195af);
pub const CMD_ADD_CLIPBOARD_IMAGE: StatusName = StatusName(0x2195b0);
pub const CMD_OPEN_DIRECTORY: StatusName = StatusName(0x2195b1);
pub const CMD_REINIT_DEV_DIR: StatusName = StatusName(0x2195b2);
pub const CMD_LAUNCH_WAY_OUT: StatusName = StatusName(0x2195b3);
pub const CMD_STOP: StatusName = StatusName(0x2195b4);

// --- event tags (host writes payload field 0; engine validates) ---
pub const EVT_START: EventTag = EventTag(0x10ccca);
pub const EVT_TIMEOUT: EventTag = EventTag(0x10cccc);
pub const EVT_MENU_ACCESS_WANTED: EventTag = EventTag(0x10cccd);
pub const EVT_MENU_ACCESS_UNWANTED: EventTag = EventTag(0x10ccce);
pub const EVT_LEAPTOFROGANS: EventTag = EventTag(0x10ccd1);
pub const EVT_CHANGE_LAYOUT: EventTag = EventTag(0x10ccd2);
pub const EVT_QUIT: EventTag = EventTag(0x10ccd3);

// --- the 17 calls ---
// commands (engine → host): `Pop<Payload>`
pub type UpdateImagesPop = Pop<update_images::UpdateImages>;
pub type UpdateZoomPop = Pop<update_zoom::UpdateZoom>;
pub type UpdateLayoutPop = Pop<update_layout::UpdateLayout>;
pub type UpdateDirectionalityPop = Pop<update_directionality::UpdateDirectionality>;
pub type AddClipboardTextPop = Pop<add_clipboard_text::AddClipboardText>;
pub type AddClipboardImagePop = Pop<add_clipboard_image::AddClipboardImage>;
pub type OpenDirectoryPop = Pop<open_directory::OpenDirectory>;
pub type ReinitializeDevelopersDirectoryPop = Pop<StatusName>;
pub type LaunchWayOutPop = Pop<launch_way_out::LaunchWayOut>;
pub type StopPop = Pop<StatusName>;
// events (host → engine): `Report<Payload>`
pub type StartReport = Report<event_start::EventStart>;
pub type TimeoutReport = Report<EventTag>;
pub type MenuAccessWantedReport = Report<event_menu_access_wanted::MenuAccessWanted>;
pub type MenuAccessUnwantedReport = Report<EventTag>;
pub type LeaptofrogansReport = Report<event_leaptofrogans::EventLeaptofrogans>;
pub type QuitReport = Report<EventTag>;
pub type ChangeLayoutReport = Report<event_change_layout::EventChangeLayout>;
