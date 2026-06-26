//! The host seam: where a [`Library`](crate::Library) gets its [`Fprt`] table.

use fprt_sys::Fprt;

/// A source of the live [`Fprt`] export table.
///
/// This is the *only* thing [`Library`](crate::Library) needs in order to call
/// the engine, and it carries the one genuine lifetime in the whole ABI: the
/// returned `&Fprt` borrows `self`, so the table's code pointers cannot outlive
/// whatever keeps the engine module mapped (the host owns that — e.g. the
/// `libloading::Library` inside [`LibloadingHost`]).
///
/// # Safety
///
/// Implementors must guarantee that [`methods`](FprtHost::methods) returns a
/// table of **valid, correctly-typed** pointers into a real FPRT engine, and
/// that every one of them stays callable for as long as `&self` is borrowed.
/// Discharging this obligation is the entire job of an implementation; once it
/// holds, every call `Library` makes through the table is sound.
pub unsafe trait FprtHost: Send + Sync {
    /// The live export table. The borrow ties the pointers' validity to `self`.
    fn methods(&self) -> &Fprt;
}

/// A host backed by a dynamically-loaded engine module (`libloading`).
///
/// Resolves the exports once at construction and owns the module, so the
/// resolved pointers stay valid for the host's lifetime — which is exactly the
/// safety obligation of [`FprtHost`], discharged here next to the `get` calls.
#[cfg(feature = "libloading")]
pub struct LibloadingHost {
    fprt: Fprt,
    // Owns the mapping. `fprt` is just `Copy` code pointers with no `Drop`; this
    // field is what actually keeps them valid, and it unmaps only when the host
    // drops — after `Library`'s `finalize` has already run.
    _module: libloading::Library,
}

#[cfg(feature = "libloading")]
impl LibloadingHost {
    /// Load the engine module at `path` and resolve its exports.
    ///
    /// The symbol strings are the documented `fprt.dll` export names: library /
    /// conductor exports are `fprt_<area>_<name>`, and UI exports follow
    /// `fprt_ui_<component>_command_<name>_pop` (engine → host) and
    /// `fprt_ui_<component>_event_<name>_report` (host → engine). They should be
    /// verified against `fprt.def`; a typo surfaces only at load time.
    pub fn open(path: impl AsRef<std::ffi::OsStr>) -> Result<Self, libloading::Error> {
        // One row per export: `field => b"symbol\0"`. The completeness check is
        // `fprt_sys::Fprt` itself — a missing field fails to compile.
        macro_rules! resolve {
            ($module:expr, { $($field:ident => $symbol:literal),+ $(,)? }) => {
                Fprt { $($field: *$module.get($symbol)?),+ }
            };
        }

        // SAFETY: loading an arbitrary module and trusting its exports to match
        // the `fprt_sys` typedefs is the unavoidable unsafe core of dynamic
        // linking; we resolve every symbol the `Fprt` table requires and copy
        // the pointers out while `module` (kept in `_module`) stays mapped.
        unsafe {
            let module = libloading::Library::new(path)?;
            let fprt = resolve!(module, {
                // --- library lifecycle ---
                library_initialize => b"fprt_library_initialize\0",
                library_finalize => b"fprt_library_finalize\0",
                library_is_initialized => b"fprt_library_is_initialized\0",
                library_report_allocated_arguments => b"fprt_library_report_allocated_arguments\0",
                library_free_allocated_arguments => b"fprt_library_free_allocated_arguments\0",

                // --- conductor ---
                conductor_start => b"fprt_conductor_start\0",
                conductor_stop => b"fprt_conductor_stop\0",
                conductor_get_next_command => b"fprt_conductor_get_next_command\0",
                conductor_sync_enter => b"fprt_conductor_sync_enter\0",
                conductor_sync_leave => b"fprt_conductor_sync_leave\0",
                conductor_sleep_enter => b"fprt_conductor_sleep_enter\0",
                conductor_sleep_leave => b"fprt_conductor_sleep_leave\0",

                // --- ui::application ---
                application_update_images => b"fprt_ui_application_command_update_images_pop\0",
                application_update_zoom => b"fprt_ui_application_command_update_zoom_pop\0",
                application_update_layout => b"fprt_ui_application_command_update_layout_pop\0",
                application_update_directionality => b"fprt_ui_application_command_update_directionality_pop\0",
                application_add_clipboard_text => b"fprt_ui_application_command_add_clipboard_text_pop\0",
                application_add_clipboard_image => b"fprt_ui_application_command_add_clipboard_image_pop\0",
                application_open_directory => b"fprt_ui_application_command_open_directory_pop\0",
                application_reinitialize_developers_directory => b"fprt_ui_application_command_reinitialize_developers_directory_pop\0",
                application_launch_way_out => b"fprt_ui_application_command_launch_way_out_pop\0",
                application_stop => b"fprt_ui_application_command_stop_pop\0",
                application_start => b"fprt_ui_application_event_start_report\0",
                application_timeout => b"fprt_ui_application_event_timeout_report\0",
                application_menu_access_wanted => b"fprt_ui_application_event_menu_access_wanted_report\0",
                application_menu_access_unwanted => b"fprt_ui_application_event_menu_access_unwanted_report\0",
                application_leaptofrogans => b"fprt_ui_application_event_leaptofrogans_report\0",
                application_quit => b"fprt_ui_application_event_quit_report\0",
                application_change_layout => b"fprt_ui_application_event_change_layout_report\0",

                // --- ui::sitehandler ---
                sitehandler_open => b"fprt_ui_sitehandler_command_open_pop\0",
                sitehandler_close => b"fprt_ui_sitehandler_command_close_pop\0",
                sitehandler_show => b"fprt_ui_sitehandler_command_show_pop\0",
                sitehandler_hide => b"fprt_ui_sitehandler_command_hide_pop\0",
                sitehandler_begin_animation_inprogress => b"fprt_ui_sitehandler_command_begin_animation_inprogress_pop\0",
                sitehandler_end_animation_inprogress => b"fprt_ui_sitehandler_command_end_animation_inprogress_pop\0",
                sitehandler_push => b"fprt_ui_sitehandler_command_push_pop\0",
                sitehandler_update_layout => b"fprt_ui_sitehandler_command_update_layout_pop\0",
                sitehandler_update_visual => b"fprt_ui_sitehandler_command_update_visual_pop\0",
                sitehandler_button_triggered => b"fprt_ui_sitehandler_event_button_triggered_report\0",
                sitehandler_force_close => b"fprt_ui_sitehandler_event_force_close_report\0",

                // --- ui::menu ---
                menu_open => b"fprt_ui_menu_command_open_pop\0",
                menu_show => b"fprt_ui_menu_command_show_pop\0",
                menu_push => b"fprt_ui_menu_command_push_pop\0",
                menu_hide => b"fprt_ui_menu_command_hide_pop\0",
                menu_close => b"fprt_ui_menu_command_close_pop\0",
                menu_update_visual => b"fprt_ui_menu_command_update_visual_pop\0",
                menu_update_layout => b"fprt_ui_menu_command_update_layout_pop\0",
                menu_button_triggered => b"fprt_ui_menu_event_button_triggered_report\0",

                // --- ui::favorites ---
                favorites_open => b"fprt_ui_favorites_command_open_pop\0",
                favorites_show => b"fprt_ui_favorites_command_show_pop\0",
                favorites_push => b"fprt_ui_favorites_command_push_pop\0",
                favorites_hide => b"fprt_ui_favorites_command_hide_pop\0",
                favorites_close => b"fprt_ui_favorites_command_close_pop\0",
                favorites_update_labels => b"fprt_ui_favorites_command_update_labels_pop\0",
                favorites_update_addresses => b"fprt_ui_favorites_command_update_addresses_pop\0",
                favorites_open_event => b"fprt_ui_favorites_event_open_report\0",
                favorites_remove => b"fprt_ui_favorites_event_remove_report\0",
                favorites_remove_all => b"fprt_ui_favorites_event_remove_all_report\0",
                favorites_cancel => b"fprt_ui_favorites_event_cancel_report\0",

                // --- ui::recentlyvisited ---
                recentlyvisited_open => b"fprt_ui_recentlyvisited_command_open_pop\0",
                recentlyvisited_show => b"fprt_ui_recentlyvisited_command_show_pop\0",
                recentlyvisited_push => b"fprt_ui_recentlyvisited_command_push_pop\0",
                recentlyvisited_hide => b"fprt_ui_recentlyvisited_command_hide_pop\0",
                recentlyvisited_close => b"fprt_ui_recentlyvisited_command_close_pop\0",
                recentlyvisited_update_labels => b"fprt_ui_recentlyvisited_command_update_labels_pop\0",
                recentlyvisited_update_addresses => b"fprt_ui_recentlyvisited_command_update_addresses_pop\0",
                recentlyvisited_open_event => b"fprt_ui_recentlyvisited_event_open_report\0",
                recentlyvisited_delete => b"fprt_ui_recentlyvisited_event_delete_report\0",
                recentlyvisited_delete_all => b"fprt_ui_recentlyvisited_event_delete_all_report\0",
                recentlyvisited_cancel => b"fprt_ui_recentlyvisited_event_cancel_report\0",

                // --- ui::inputfa ---
                inputfa_open => b"fprt_ui_inputfa_command_open_pop\0",
                inputfa_show => b"fprt_ui_inputfa_command_show_pop\0",
                inputfa_push => b"fprt_ui_inputfa_command_push_pop\0",
                inputfa_hide => b"fprt_ui_inputfa_command_hide_pop\0",
                inputfa_close => b"fprt_ui_inputfa_command_close_pop\0",
                inputfa_update_error_clear => b"fprt_ui_inputfa_command_update_error_clear_pop\0",
                inputfa_update_address => b"fprt_ui_inputfa_command_update_address_pop\0",
                inputfa_update_error_raise => b"fprt_ui_inputfa_command_update_error_raise_pop\0",
                inputfa_update_labels => b"fprt_ui_inputfa_command_update_labels_pop\0",
                inputfa_change => b"fprt_ui_inputfa_event_change_report\0",
                inputfa_ok => b"fprt_ui_inputfa_event_ok_report\0",
                inputfa_cancel => b"fprt_ui_inputfa_event_cancel_report\0",

                // --- ui::blocked ---
                blocked_open => b"fprt_ui_blocked_command_open_pop\0",
                blocked_show => b"fprt_ui_blocked_command_show_pop\0",
                blocked_push => b"fprt_ui_blocked_command_push_pop\0",
                blocked_hide => b"fprt_ui_blocked_command_hide_pop\0",
                blocked_close => b"fprt_ui_blocked_command_close_pop\0",
                blocked_update_labels => b"fprt_ui_blocked_command_update_labels_pop\0",
                blocked_update_addresses => b"fprt_ui_blocked_command_update_addresses_pop\0",
                blocked_remove => b"fprt_ui_blocked_event_remove_report\0",
                blocked_remove_all => b"fprt_ui_blocked_event_remove_all_report\0",
                blocked_cancel => b"fprt_ui_blocked_event_cancel_report\0",

                // --- ui::devtools ---
                devtools_open => b"fprt_ui_devtools_command_open_pop\0",
                devtools_show => b"fprt_ui_devtools_command_show_pop\0",
                devtools_push => b"fprt_ui_devtools_command_push_pop\0",
                devtools_hide => b"fprt_ui_devtools_command_hide_pop\0",
                devtools_close => b"fprt_ui_devtools_command_close_pop\0",
                devtools_update_labels => b"fprt_ui_devtools_command_update_labels_pop\0",
                devtools_update_addresses => b"fprt_ui_devtools_command_update_addresses_pop\0",
                devtools_inspect => b"fprt_ui_devtools_event_inspect_report\0",
                devtools_cancel => b"fprt_ui_devtools_event_cancel_report\0",

                // --- ui::recovery ---
                recovery_open => b"fprt_ui_recovery_command_open_pop\0",
                recovery_show => b"fprt_ui_recovery_command_show_pop\0",
                recovery_hide => b"fprt_ui_recovery_command_hide_pop\0",
                recovery_close => b"fprt_ui_recovery_command_close_pop\0",
                recovery_update_labels => b"fprt_ui_recovery_command_update_labels_pop\0",
                recovery_update_addresses => b"fprt_ui_recovery_command_update_addresses_pop\0",
                recovery_open_event => b"fprt_ui_recovery_event_open_report\0",
                recovery_cancel => b"fprt_ui_recovery_event_cancel_report\0",

                // --- ui::zoom ---
                zoom_open => b"fprt_ui_zoom_command_open_pop\0",
                zoom_show => b"fprt_ui_zoom_command_show_pop\0",
                zoom_push => b"fprt_ui_zoom_command_push_pop\0",
                zoom_hide => b"fprt_ui_zoom_command_hide_pop\0",
                zoom_close => b"fprt_ui_zoom_command_close_pop\0",
                zoom_update_labels => b"fprt_ui_zoom_command_update_labels_pop\0",
                zoom_ok => b"fprt_ui_zoom_event_ok_report\0",
                zoom_cancel => b"fprt_ui_zoom_event_cancel_report\0",

                // --- ui::update ---
                update_open => b"fprt_ui_update_command_open_pop\0",
                update_show => b"fprt_ui_update_command_show_pop\0",
                update_push => b"fprt_ui_update_command_push_pop\0",
                update_hide => b"fprt_ui_update_command_hide_pop\0",
                update_close => b"fprt_ui_update_command_close_pop\0",
                update_update_labels => b"fprt_ui_update_command_update_labels_pop\0",
                update_update_data => b"fprt_ui_update_command_update_data_pop\0",
                update_cancel => b"fprt_ui_update_event_cancel_report\0",

                // --- ui::pad (commands only) ---
                pad_open => b"fprt_ui_pad_command_open_pop\0",
                pad_show => b"fprt_ui_pad_command_show_pop\0",
                pad_hide => b"fprt_ui_pad_command_hide_pop\0",
                pad_close => b"fprt_ui_pad_command_close_pop\0",
                pad_begin_animation => b"fprt_ui_pad_command_begin_animation_inprogress_pop\0",
                pad_end_animation => b"fprt_ui_pad_command_end_animation_inprogress_pop\0",
                pad_update_layout => b"fprt_ui_pad_command_update_layout_pop\0",

                // --- ui::language ---
                language_open => b"fprt_ui_language_command_open_pop\0",
                language_show => b"fprt_ui_language_command_show_pop\0",
                language_push => b"fprt_ui_language_command_push_pop\0",
                language_hide => b"fprt_ui_language_command_hide_pop\0",
                language_close => b"fprt_ui_language_command_close_pop\0",
                language_update_labels => b"fprt_ui_language_command_update_labels_pop\0",
                language_update_list => b"fprt_ui_language_command_update_list_pop\0",
                language_ok => b"fprt_ui_language_event_ok_report\0",
                language_cancel => b"fprt_ui_language_event_cancel_report\0",

                // --- ui::leaptofrogans ---
                leaptofrogans_open => b"fprt_ui_leaptofrogans_command_open_pop\0",
                leaptofrogans_show => b"fprt_ui_leaptofrogans_command_show_pop\0",
                leaptofrogans_push => b"fprt_ui_leaptofrogans_command_push_pop\0",
                leaptofrogans_hide => b"fprt_ui_leaptofrogans_command_hide_pop\0",
                leaptofrogans_close => b"fprt_ui_leaptofrogans_command_close_pop\0",
                leaptofrogans_update_labels => b"fprt_ui_leaptofrogans_command_update_labels_pop\0",
                leaptofrogans_update_address => b"fprt_ui_leaptofrogans_command_update_address_pop\0",
                leaptofrogans_confirm => b"fprt_ui_leaptofrogans_event_confirm_report\0",
                leaptofrogans_cancel => b"fprt_ui_leaptofrogans_event_cancel_report\0",
                leaptofrogans_block => b"fprt_ui_leaptofrogans_event_block_report\0",
                leaptofrogans_purge => b"fprt_ui_leaptofrogans_event_purge_report\0",
                leaptofrogans_close_event => b"fprt_ui_leaptofrogans_event_close_report\0",

                // --- ui::legalinformation ---
                legalinformation_open => b"fprt_ui_legalinformation_command_open_pop\0",
                legalinformation_show => b"fprt_ui_legalinformation_command_show_pop\0",
                legalinformation_push => b"fprt_ui_legalinformation_command_push_pop\0",
                legalinformation_hide => b"fprt_ui_legalinformation_command_hide_pop\0",
                legalinformation_close => b"fprt_ui_legalinformation_command_close_pop\0",
                legalinformation_update_labels => b"fprt_ui_legalinformation_command_update_labels_pop\0",
                legalinformation_update_legal_content => b"fprt_ui_legalinformation_command_update_legal_content_pop\0",
                legalinformation_close_event => b"fprt_ui_legalinformation_event_close_report\0",

                // --- ui::inspector ---
                inspector_open => b"fprt_ui_inspector_command_open_pop\0",
                inspector_close => b"fprt_ui_inspector_command_close_pop\0",
                inspector_show => b"fprt_ui_inspector_command_show_pop\0",
                inspector_hide => b"fprt_ui_inspector_command_hide_pop\0",
                inspector_push => b"fprt_ui_inspector_command_push_pop\0",
                inspector_update_address => b"fprt_ui_inspector_command_update_address_pop\0",
                inspector_update_status => b"fprt_ui_inspector_command_update_status_pop\0",
                inspector_update_labels => b"fprt_ui_inspector_command_update_labels_pop\0",
                inspector_update_steps_labels => b"fprt_ui_inspector_command_update_steps_labels_pop\0",
                inspector_update_content_labels => b"fprt_ui_inspector_command_update_content_labels_pop\0",
                inspector_update_content_viewer => b"fprt_ui_inspector_command_update_content_viewer_pop\0",
                inspector_update_sync => b"fprt_ui_inspector_command_update_sync_pop\0",
                inspector_step_selected => b"fprt_ui_inspector_event_step_selected_report\0",
                inspector_content_selected => b"fprt_ui_inspector_event_content_selected_report\0",
                inspector_synchronize => b"fprt_ui_inspector_event_synchronize_report\0",
                inspector_change_autosync => b"fprt_ui_inspector_event_change_autosync_report\0",
                inspector_rerun => b"fprt_ui_inspector_event_rerun_report\0",
                inspector_close_event => b"fprt_ui_inspector_event_close_report\0",
            });
            Ok(LibloadingHost { fprt, _module: module })
        }
    }
}

// SAFETY: `fprt` was resolved from `_module`, which this struct owns and keeps
// mapped for as long as `&self` lives, so the pointers remain valid.
#[cfg(feature = "libloading")]
unsafe impl FprtHost for LibloadingHost {
    fn methods(&self) -> &Fprt {
        &self.fprt
    }
}
