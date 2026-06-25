#![cfg_attr(not(test), no_std)]

//! `fprt-sys` — exact, raw declarations of the Frogans Player C ABI.
//!
//! Pure contract: `#[repr(C)]` types (one per file), status constants, and the
//! per-export function typedefs (one module per export), plus the [`Fprt`] table
//! that enumerates every export. No behavior.
//!
//! Both sides of the seam depend on this crate:
//!   * a DLL implementation proves completeness + correct typing by constructing
//!     one [`Fprt`] value — a missing or mistyped field is a compile error;
//!   * a host types its dynamically-loaded function pointers with these typedefs.
//!
//! Layouts and codes come from the reverse-engineering notes in
//! `../../re/work/notes/cabi/`. Covered so far: the library lifecycle, and the
//! `start` / `stop` conductor exports.

pub mod conductor;
pub mod ctx;
pub mod deployment_mode;
pub mod devtools_support;
pub mod exit_button_support;
pub mod image_format;
pub mod library;
pub mod library_version;
pub mod mem;
pub mod nature;
pub mod reserved_flag;
pub mod start_information;
pub mod ui;
pub mod ustring;

use crate::conductor::get_next_command::FprtConductorGetNextCommand;
use crate::conductor::sleep_enter::FprtConductorSleepEnter;
use crate::conductor::sleep_leave::FprtConductorSleepLeave;
use crate::conductor::start::FprtConductorStart;
use crate::conductor::stop::FprtConductorStop;
use crate::conductor::sync_enter::FprtConductorSyncEnter;
use crate::conductor::sync_leave::FprtConductorSyncLeave;
use crate::library::finalize::FprtLibraryFinalize;
use crate::library::free_allocated_arguments::FprtLibraryFreeAllocatedArguments;
use crate::library::initialize::FprtLibraryInitialize;
use crate::library::is_initialized::FprtLibraryIsInitialized;
use crate::library::report_allocated_arguments::FprtLibraryReportAllocatedArguments;

/// The complete table of exports — the enumeration.
///
/// Constructing an `Fprt` requires every field, each correctly typed, so it is
/// both the completeness + type check for an implementation (DLL side) and the
/// call table for a host that has loaded the symbols. Grows toward all 179.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Fprt {
    pub library_initialize: FprtLibraryInitialize,
    pub library_finalize: FprtLibraryFinalize,
    pub library_is_initialized: FprtLibraryIsInitialized,
    pub library_report_allocated_arguments: FprtLibraryReportAllocatedArguments,
    pub library_free_allocated_arguments: FprtLibraryFreeAllocatedArguments,
    pub conductor_start: FprtConductorStart,
    pub conductor_stop: FprtConductorStop,
    pub conductor_get_next_command: FprtConductorGetNextCommand,
    pub conductor_sync_enter: FprtConductorSyncEnter,
    pub conductor_sync_leave: FprtConductorSyncLeave,
    pub conductor_sleep_enter: FprtConductorSleepEnter,
    pub conductor_sleep_leave: FprtConductorSleepLeave,

    // ui::application (17)
    pub application_update_images: crate::ui::application::UpdateImagesPop,
    pub application_update_zoom: crate::ui::application::UpdateZoomPop,
    pub application_update_layout: crate::ui::application::UpdateLayoutPop,
    pub application_update_directionality: crate::ui::application::UpdateDirectionalityPop,
    pub application_add_clipboard_text: crate::ui::application::AddClipboardTextPop,
    pub application_add_clipboard_image: crate::ui::application::AddClipboardImagePop,
    pub application_open_directory: crate::ui::application::OpenDirectoryPop,
    pub application_reinitialize_developers_directory:
        crate::ui::application::ReinitializeDevelopersDirectoryPop,
    pub application_launch_way_out: crate::ui::application::LaunchWayOutPop,
    pub application_stop: crate::ui::application::StopPop,
    pub application_start: crate::ui::application::StartReport,
    pub application_timeout: crate::ui::application::TimeoutReport,
    pub application_menu_access_wanted: crate::ui::application::MenuAccessWantedReport,
    pub application_menu_access_unwanted: crate::ui::application::MenuAccessUnwantedReport,
    pub application_leaptofrogans: crate::ui::application::LeaptofrogansReport,
    pub application_quit: crate::ui::application::QuitReport,
    pub application_change_layout: crate::ui::application::ChangeLayoutReport,
}
