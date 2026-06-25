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

    // ui::sitehandler (11)
    pub sitehandler_open: crate::ui::sitehandler::OpenPop,
    pub sitehandler_close: crate::ui::sitehandler::ClosePop,
    pub sitehandler_show: crate::ui::sitehandler::ShowPop,
    pub sitehandler_hide: crate::ui::sitehandler::HidePop,
    pub sitehandler_begin_animation_inprogress:
        crate::ui::sitehandler::BeginAnimationInprogressPop,
    pub sitehandler_end_animation_inprogress: crate::ui::sitehandler::EndAnimationInprogressPop,
    pub sitehandler_push: crate::ui::sitehandler::PushPop,
    pub sitehandler_update_layout: crate::ui::sitehandler::UpdateLayoutPop,
    pub sitehandler_update_visual: crate::ui::sitehandler::UpdateVisualPop,
    pub sitehandler_button_triggered: crate::ui::sitehandler::ButtonTriggeredReport,
    pub sitehandler_force_close: crate::ui::sitehandler::ForceCloseReport,

    // ui::menu (8)
    pub menu_open: crate::ui::menu::OpenPop,
    pub menu_show: crate::ui::menu::ShowPop,
    pub menu_push: crate::ui::menu::PushPop,
    pub menu_hide: crate::ui::menu::HidePop,
    pub menu_close: crate::ui::menu::ClosePop,
    pub menu_update_visual: crate::ui::menu::UpdateVisualPop,
    pub menu_update_layout: crate::ui::menu::UpdateLayoutPop,
    pub menu_button_triggered: crate::ui::menu::ButtonTriggeredReport,

    // ui::favorites (11)
    pub favorites_open: crate::ui::favorites::OpenPop,
    pub favorites_show: crate::ui::favorites::ShowPop,
    pub favorites_push: crate::ui::favorites::PushPop,
    pub favorites_hide: crate::ui::favorites::HidePop,
    pub favorites_close: crate::ui::favorites::ClosePop,
    pub favorites_update_labels: crate::ui::favorites::UpdateLabelsPop,
    pub favorites_update_addresses: crate::ui::favorites::UpdateAddressesPop,
    pub favorites_open_event: crate::ui::favorites::OpenReport,
    pub favorites_remove: crate::ui::favorites::RemoveReport,
    pub favorites_remove_all: crate::ui::favorites::RemoveAllReport,
    pub favorites_cancel: crate::ui::favorites::CancelReport,

    // ui::recentlyvisited (11)
    pub recentlyvisited_open: crate::ui::recentlyvisited::OpenPop,
    pub recentlyvisited_show: crate::ui::recentlyvisited::ShowPop,
    pub recentlyvisited_push: crate::ui::recentlyvisited::PushPop,
    pub recentlyvisited_hide: crate::ui::recentlyvisited::HidePop,
    pub recentlyvisited_close: crate::ui::recentlyvisited::ClosePop,
    pub recentlyvisited_update_labels: crate::ui::recentlyvisited::UpdateLabelsPop,
    pub recentlyvisited_update_addresses: crate::ui::recentlyvisited::UpdateAddressesPop,
    pub recentlyvisited_open_event: crate::ui::recentlyvisited::OpenReport,
    pub recentlyvisited_delete: crate::ui::recentlyvisited::DeleteReport,
    pub recentlyvisited_delete_all: crate::ui::recentlyvisited::DeleteAllReport,
    pub recentlyvisited_cancel: crate::ui::recentlyvisited::CancelReport,

    // ui::inputfa (12)
    pub inputfa_open: crate::ui::inputfa::OpenPop,
    pub inputfa_show: crate::ui::inputfa::ShowPop,
    pub inputfa_push: crate::ui::inputfa::PushPop,
    pub inputfa_hide: crate::ui::inputfa::HidePop,
    pub inputfa_close: crate::ui::inputfa::ClosePop,
    pub inputfa_update_error_clear: crate::ui::inputfa::UpdateErrorClearPop,
    pub inputfa_update_address: crate::ui::inputfa::UpdateAddressPop,
    pub inputfa_update_error_raise: crate::ui::inputfa::UpdateErrorRaisePop,
    pub inputfa_update_labels: crate::ui::inputfa::UpdateLabelsPop,
    pub inputfa_change: crate::ui::inputfa::ChangeReport,
    pub inputfa_ok: crate::ui::inputfa::OkReport,
    pub inputfa_cancel: crate::ui::inputfa::CancelReport,

    // ui::blocked (10)
    pub blocked_open: crate::ui::blocked::OpenPop,
    pub blocked_show: crate::ui::blocked::ShowPop,
    pub blocked_push: crate::ui::blocked::PushPop,
    pub blocked_hide: crate::ui::blocked::HidePop,
    pub blocked_close: crate::ui::blocked::ClosePop,
    pub blocked_update_labels: crate::ui::blocked::UpdateLabelsPop,
    pub blocked_update_addresses: crate::ui::blocked::UpdateAddressesPop,
    pub blocked_remove: crate::ui::blocked::RemoveReport,
    pub blocked_remove_all: crate::ui::blocked::RemoveAllReport,
    pub blocked_cancel: crate::ui::blocked::CancelReport,

    // ui::devtools (9)
    pub devtools_open: crate::ui::devtools::OpenPop,
    pub devtools_show: crate::ui::devtools::ShowPop,
    pub devtools_push: crate::ui::devtools::PushPop,
    pub devtools_hide: crate::ui::devtools::HidePop,
    pub devtools_close: crate::ui::devtools::ClosePop,
    pub devtools_update_labels: crate::ui::devtools::UpdateLabelsPop,
    pub devtools_update_addresses: crate::ui::devtools::UpdateAddressesPop,
    pub devtools_inspect: crate::ui::devtools::InspectReport,
    pub devtools_cancel: crate::ui::devtools::CancelReport,

    // ui::recovery (8) — command `open` + event `open` ⇒ `_open_event` suffix
    pub recovery_open: crate::ui::recovery::OpenPop,
    pub recovery_show: crate::ui::recovery::ShowPop,
    pub recovery_hide: crate::ui::recovery::HidePop,
    pub recovery_close: crate::ui::recovery::ClosePop,
    pub recovery_update_labels: crate::ui::recovery::UpdateLabelsPop,
    pub recovery_update_addresses: crate::ui::recovery::UpdateAddressesPop,
    pub recovery_open_event: crate::ui::recovery::OpenReport,
    pub recovery_cancel: crate::ui::recovery::CancelReport,

    // ui::zoom (8)
    pub zoom_open: crate::ui::zoom::OpenPop,
    pub zoom_show: crate::ui::zoom::ShowPop,
    pub zoom_push: crate::ui::zoom::PushPop,
    pub zoom_hide: crate::ui::zoom::HidePop,
    pub zoom_close: crate::ui::zoom::ClosePop,
    pub zoom_update_labels: crate::ui::zoom::UpdateLabelsPop,
    pub zoom_ok: crate::ui::zoom::OkReport,
    pub zoom_cancel: crate::ui::zoom::CancelReport,

    // ui::update (8)
    pub update_open: crate::ui::update::OpenPop,
    pub update_show: crate::ui::update::ShowPop,
    pub update_push: crate::ui::update::PushPop,
    pub update_hide: crate::ui::update::HidePop,
    pub update_close: crate::ui::update::ClosePop,
    pub update_update_labels: crate::ui::update::UpdateLabelsPop,
    pub update_update_data: crate::ui::update::UpdateDataPop,
    pub update_cancel: crate::ui::update::CancelReport,

    // ui::pad (7) — commands only, no events
    pub pad_open: crate::ui::pad::OpenPop,
    pub pad_show: crate::ui::pad::ShowPop,
    pub pad_hide: crate::ui::pad::HidePop,
    pub pad_close: crate::ui::pad::ClosePop,
    pub pad_begin_animation: crate::ui::pad::BeginAnimationPop,
    pub pad_end_animation: crate::ui::pad::EndAnimationPop,
    pub pad_update_layout: crate::ui::pad::UpdateLayoutPop,

    // ui::language (9)
    pub language_open: crate::ui::language::OpenPop,
    pub language_show: crate::ui::language::ShowPop,
    pub language_push: crate::ui::language::PushPop,
    pub language_hide: crate::ui::language::HidePop,
    pub language_close: crate::ui::language::ClosePop,
    pub language_update_labels: crate::ui::language::UpdateLabelsPop,
    pub language_update_list: crate::ui::language::UpdateListPop,
    pub language_ok: crate::ui::language::OkReport,
    pub language_cancel: crate::ui::language::CancelReport,

    // ui::leaptofrogans (12) — command `close` + event `close` ⇒ `_close_event` suffix
    pub leaptofrogans_open: crate::ui::leaptofrogans::OpenPop,
    pub leaptofrogans_show: crate::ui::leaptofrogans::ShowPop,
    pub leaptofrogans_push: crate::ui::leaptofrogans::PushPop,
    pub leaptofrogans_hide: crate::ui::leaptofrogans::HidePop,
    pub leaptofrogans_close: crate::ui::leaptofrogans::ClosePop,
    pub leaptofrogans_update_labels: crate::ui::leaptofrogans::UpdateLabelsPop,
    pub leaptofrogans_update_address: crate::ui::leaptofrogans::UpdateAddressPop,
    pub leaptofrogans_confirm: crate::ui::leaptofrogans::ConfirmReport,
    pub leaptofrogans_cancel: crate::ui::leaptofrogans::CancelReport,
    pub leaptofrogans_block: crate::ui::leaptofrogans::BlockReport,
    pub leaptofrogans_purge: crate::ui::leaptofrogans::PurgeReport,
    pub leaptofrogans_close_event: crate::ui::leaptofrogans::CloseReport,

    // ui::legalinformation (8) — command `close` + event `close` ⇒ `_close_event` suffix
    pub legalinformation_open: crate::ui::legalinformation::OpenPop,
    pub legalinformation_show: crate::ui::legalinformation::ShowPop,
    pub legalinformation_push: crate::ui::legalinformation::PushPop,
    pub legalinformation_hide: crate::ui::legalinformation::HidePop,
    pub legalinformation_close: crate::ui::legalinformation::ClosePop,
    pub legalinformation_update_labels: crate::ui::legalinformation::UpdateLabelsPop,
    pub legalinformation_update_legal_content:
        crate::ui::legalinformation::UpdateLegalContentPop,
    pub legalinformation_close_event: crate::ui::legalinformation::CloseReport,

    // ui::inspector (18) — command `close` + event `close` ⇒ `_close_event` suffix
    pub inspector_open: crate::ui::inspector::OpenPop,
    pub inspector_close: crate::ui::inspector::ClosePop,
    pub inspector_show: crate::ui::inspector::ShowPop,
    pub inspector_hide: crate::ui::inspector::HidePop,
    pub inspector_push: crate::ui::inspector::PushPop,
    pub inspector_update_address: crate::ui::inspector::UpdateAddressPop,
    pub inspector_update_status: crate::ui::inspector::UpdateStatusPop,
    pub inspector_update_labels: crate::ui::inspector::UpdateLabelsPop,
    pub inspector_update_steps_labels: crate::ui::inspector::UpdateStepsLabelsPop,
    pub inspector_update_content_labels: crate::ui::inspector::UpdateContentLabelsPop,
    pub inspector_update_content_viewer: crate::ui::inspector::UpdateContentViewerPop,
    pub inspector_update_sync: crate::ui::inspector::UpdateSyncPop,
    pub inspector_step_selected: crate::ui::inspector::StepSelectedReport,
    pub inspector_content_selected: crate::ui::inspector::ContentSelectedReport,
    pub inspector_synchronize: crate::ui::inspector::SynchronizeReport,
    pub inspector_change_autosync: crate::ui::inspector::ChangeAutosyncReport,
    pub inspector_rerun: crate::ui::inspector::RerunReport,
    pub inspector_close_event: crate::ui::inspector::CloseReport,
}
