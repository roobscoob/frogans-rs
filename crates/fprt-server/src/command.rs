//! Engine→host command transport: the [`CommandEncode`] seam each `_pop`
//! trampoline rides, and [`status_name`] — the `get_next_command` map.
//!
//! The mirror of the client's `conductor::command`. There, each command type
//! carries a `CommandPayload` that *decodes* a popped raw payload; here each
//! carries a [`CommandEncode`] that *encodes* the staged [`Command`] back into its
//! raw `#[repr(C)]` form. The generic `pop::<C>` trampoline (in [`engine`]) is the
//! single body all of them share — `C` only supplies the id, the raw type, and the
//! one-line encode.
//!
//! Four shapes cover every command: [`marker!`] (no payload, `Raw = StatusName`),
//! [`typed!`] (`to_raw(&self)` — scalar / single-pooled, no pool arg), [`pooled!`]
//! (`to_raw(&self, pool)` — list payloads), and [`id_carrier!`] (a lifecycle command
//! whose whole payload is the engine id it targets). Five complex encodes
//! (`UpdateImages`, menu/sitehandler `UpdateVisual`, `UpdateList`,
//! `UpdateLegalContent`) await their `to_raw` half in fprt-core and stay
//! `unimpl_pop` for now.
//!
//! [`engine`]: crate::engine

use fprt_core::Command;
use fprt_core::component::{
    application, blocked, devtools, favorites, inputfa, inspector, language, leaptofrogans,
    legalinformation, menu, pad, recentlyvisited, recovery, sitehandler, update, zoom,
};
use fprt_sys::ui::StatusName;
use fprt_sys::ui::{
    application as app_id, blocked as blocked_id, devtools as devtools_id, favorites as favorites_id,
    inputfa as inputfa_id, inspector as inspector_id, language as language_id,
    leaptofrogans as leaptofrogans_id, legalinformation as legal_id, menu as menu_id, pad as pad_id,
    recentlyvisited as recently_id, recovery as recovery_id, sitehandler as site_id_mod,
    update as update_id, zoom as zoom_id,
};

use crate::encoder::CallPool;

/// What a `_pop` trampoline needs to turn the next staged [`Command`] into the raw
/// payload the host reads. One impl per command (or marker); `pop::<C>` does the
/// rest.
pub(crate) trait CommandEncode {
    /// The `0x2195xx` operation-result tag stamped into the payload's field 0 and
    /// reported by `get_next_command` — the command's class id.
    const ID: StatusName;
    /// The `0x17`/`0x18` error-block base for this call (error-code DB §5.2), used
    /// to form framework-level reject codes (`BASE + offset`) before the engine
    /// runs (e.g. `BASE + 0` = bad context).
    const BASE: u32;
    /// Human breadcrumb stamped onto a caught panic (`"ui.menu.command_open"`).
    const OP: &'static str;
    /// The raw `#[repr(C)]` payload this command pops into.
    type Raw;
    /// Encode the staged command into its raw payload. Scalars/markers ignore
    /// `pool`; a list payload materializes it (`pool.arena()`) only to allocate its
    /// descriptor array — so a no-data command leaves the pool empty (`EMPTY`
    /// handle). Called only with the matching [`Command`] variant (`pop::<C>` peeked
    /// its id first).
    fn encode(command: Command, pool: &CallPool) -> Self::Raw;
}

/// No-payload command marker (`Raw = StatusName`, field 0 = the id).
macro_rules! marker {
    ($name:ident, $id:expr, $base:expr, $op:expr) => {
        /// A no-payload command marker (engine → host); `pop::<…>` writes its id.
        pub(crate) struct $name;

        impl CommandEncode for $name {
            const ID: StatusName = $id;
            const BASE: u32 = $base;
            const OP: &'static str = $op;
            type Raw = StatusName;

            fn encode(_command: Command, _pool: &CallPool) -> StatusName {
                $id
            }
        }
    };
}

/// Data command whose `to_raw(&self)` needs no pool (scalars and single-pooled
/// payloads — the descriptor points at bytes already allocated when the command was
/// emitted). Implemented on the core payload type.
macro_rules! typed {
    ($ty:ty, $variant:path, $raw:ty, $id:expr, $base:expr, $op:expr) => {
        impl CommandEncode for $ty {
            const ID: StatusName = $id;
            const BASE: u32 = $base;
            const OP: &'static str = $op;
            type Raw = $raw;

            fn encode(command: Command, _pool: &CallPool) -> Self::Raw {
                match command {
                    $variant(payload) => payload.to_raw(),
                    other => panic!(concat!($op, ": wrong command variant: {:?}"), other),
                }
            }
        }
    };
}

/// Data command whose `to_raw(&self, pool)` allocates a descriptor array — the pool
/// is materialized here, the one place a list payload needs it.
macro_rules! pooled {
    ($ty:ty, $variant:path, $raw:ty, $id:expr, $base:expr, $op:expr) => {
        impl CommandEncode for $ty {
            const ID: StatusName = $id;
            const BASE: u32 = $base;
            const OP: &'static str = $op;
            type Raw = $raw;

            fn encode(command: Command, pool: &CallPool) -> Self::Raw {
                match command {
                    $variant(payload) => payload.to_raw(pool.arena()),
                    other => panic!(concat!($op, ": wrong command variant: {:?}"), other),
                }
            }
        }
    };
}

/// Id-carrying lifecycle command: the whole payload is `{ status_id, <field>: id }`,
/// built from the engine id the [`Command`] variant carries (inspector window /
/// Frogans site). No fprt-core payload type — the variant holds the id directly.
macro_rules! id_carrier {
    ($name:ident, $variant:path, $raw:path, $field:ident, $id:expr, $base:expr, $op:expr) => {
        /// A lifecycle command marker carrying its target id.
        pub(crate) struct $name;

        impl CommandEncode for $name {
            const ID: StatusName = $id;
            const BASE: u32 = $base;
            const OP: &'static str = $op;
            type Raw = $raw;

            fn encode(command: Command, _pool: &CallPool) -> Self::Raw {
                // Local alias: a `:path`/`:ty` metavar can't directly prefix a `{ }`
                // struct literal, but a plain ident alias can.
                type R = $raw;
                match command {
                    $variant(id) => R {
                        status_id: $id,
                        $field: id.0,
                    },
                    other => panic!(concat!($op, ": wrong command variant: {:?}"), other),
                }
            }
        }
    };
}

// ─── application ──────────────────────────────────────────────────────────────
typed!(application::UpdateZoom, Command::ApplicationUpdateZoom, fprt_sys::ui::application::update_zoom::UpdateZoom, app_id::CMD_UPDATE_ZOOM, 0x17e9db51, "ui.application.command_update_zoom");
typed!(application::UpdateLayout, Command::ApplicationUpdateLayout, fprt_sys::ui::application::update_layout::UpdateLayout, app_id::CMD_UPDATE_LAYOUT, 0x17e9df39, "ui.application.command_update_layout");
typed!(application::UpdateDirectionality, Command::ApplicationUpdateDirectionality, fprt_sys::ui::application::update_directionality::UpdateDirectionality, app_id::CMD_UPDATE_DIRECTIONALITY, 0x17e9e321, "ui.application.command_update_directionality");
typed!(application::AddClipboardText, Command::ApplicationAddClipboardText, fprt_sys::ui::application::add_clipboard_text::AddClipboardText, app_id::CMD_ADD_CLIPBOARD_TEXT, 0x17e9eaf1, "ui.application.command_add_clipboard_text");
typed!(application::AddClipboardImage, Command::ApplicationAddClipboardImage, fprt_sys::ui::application::add_clipboard_image::AddClipboardImage, app_id::CMD_ADD_CLIPBOARD_IMAGE, 0x17e9eed9, "ui.application.command_add_clipboard_image");
typed!(application::OpenDirectory, Command::ApplicationOpenDirectory, fprt_sys::ui::application::open_directory::OpenDirectory, app_id::CMD_OPEN_DIRECTORY, 0x17e9eed9, "ui.application.command_open_directory");
typed!(application::LaunchWayOut, Command::ApplicationLaunchWayOut, fprt_sys::ui::application::launch_way_out::LaunchWayOut, app_id::CMD_LAUNCH_WAY_OUT, 0x17e9f6a9, "ui.application.command_launch_way_out");
marker!(ApplicationReinitializeDevelopersDirectory, app_id::CMD_REINIT_DEV_DIR, 0x17e9f2c1, "ui.application.command_reinitialize_developers_directory");
marker!(ApplicationStop, app_id::CMD_STOP, 0x17e9fa91, "ui.application.command_stop");
pooled!(application::UpdateImages, Command::ApplicationUpdateImages, fprt_sys::ui::application::update_images::UpdateImages, app_id::CMD_UPDATE_IMAGES, 0x17e9d769, "ui.application.command_update_images");

// ─── menu ─────────────────────────────────────────────────────────────────────
marker!(MenuOpen, menu_id::CMD_OPEN, 0x18085be9, "ui.menu.command_open");
marker!(MenuShow, menu_id::CMD_SHOW, 0x180863b9, "ui.menu.command_show");
marker!(MenuPush, menu_id::CMD_PUSH, 0x180867a1, "ui.menu.command_push");
marker!(MenuHide, menu_id::CMD_HIDE, 0x18086b89, "ui.menu.command_hide");
marker!(MenuClose, menu_id::CMD_CLOSE, 0x18086f71, "ui.menu.command_close");
typed!(menu::UpdateLayout, Command::MenuUpdateLayout, fprt_sys::ui::menu::update_layout::UpdateLayout, menu_id::CMD_UPDATE_LAYOUT, 0x18085fd1, "ui.menu.command_update_layout");
pooled!(menu::UpdateVisual, Command::MenuUpdateVisual, fprt_sys::ui::menu::update_visual::UpdateVisual, menu_id::CMD_UPDATE_VISUAL, 0x18085fd1, "ui.menu.command_update_visual");

// ─── sitehandler (lifecycle = id-carriers on SiteId) ──────────────────────────
id_carrier!(SitehandlerOpen, Command::SitehandlerOpen, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_OPEN, 0x18826de9, "ui.sitehandler.command_open");
id_carrier!(SitehandlerShow, Command::SitehandlerShow, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_SHOW, 0x18828171, "ui.sitehandler.command_show");
id_carrier!(SitehandlerPush, Command::SitehandlerPush, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_PUSH, 0x18828559, "ui.sitehandler.command_push");
id_carrier!(SitehandlerHide, Command::SitehandlerHide, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_HIDE, 0x18828941, "ui.sitehandler.command_hide");
id_carrier!(SitehandlerClose, Command::SitehandlerClose, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_CLOSE, 0x18828d29, "ui.sitehandler.command_close");
id_carrier!(SitehandlerBeginAnimationInprogress, Command::SitehandlerBeginAnimationInprogress, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_BEGIN_ANIMATION_INPROGRESS, 0x188279a1, "ui.sitehandler.command_begin_animation_inprogress");
id_carrier!(SitehandlerEndAnimationInprogress, Command::SitehandlerEndAnimationInprogress, fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle, site_id, site_id_mod::CMD_END_ANIMATION_INPROGRESS, 0x18827d89, "ui.sitehandler.command_end_animation_inprogress");
typed!(sitehandler::UpdateLayout, Command::SitehandlerUpdateLayout, fprt_sys::ui::sitehandler::update_layout::UpdateLayout, site_id_mod::CMD_UPDATE_LAYOUT, 0x188271d1, "ui.sitehandler.command_update_layout");
pooled!(sitehandler::UpdateVisual, Command::SitehandlerUpdateVisual, fprt_sys::ui::sitehandler::update_visual::UpdateVisual, site_id_mod::CMD_UPDATE_VISUAL, 0x188275b9, "ui.sitehandler.command_update_visual");

// ─── favorites ────────────────────────────────────────────────────────────────
marker!(FavoritesOpen, favorites_id::CMD_OPEN, 0x1863e969, "ui.favorites.command_open");
marker!(FavoritesShow, favorites_id::CMD_SHOW, 0x1863f521, "ui.favorites.command_show");
marker!(FavoritesPush, favorites_id::CMD_PUSH, 0x1863f909, "ui.favorites.command_push");
marker!(FavoritesHide, favorites_id::CMD_HIDE, 0x1863fcf1, "ui.favorites.command_hide");
marker!(FavoritesClose, favorites_id::CMD_CLOSE, 0x186400d9, "ui.favorites.command_close");
typed!(favorites::UpdateLabels, Command::FavoritesUpdateLabels, fprt_sys::ui::favorites::labels::Labels, favorites_id::CMD_UPDATE_LABELS, 0x1863ed51, "ui.favorites.command_update_labels");
pooled!(favorites::UpdateAddresses, Command::FavoritesUpdateAddresses, fprt_sys::ui::AddressList, favorites_id::CMD_UPDATE_ADDRESSES, 0x1863f139, "ui.favorites.command_update_addresses");

// ─── recentlyvisited ──────────────────────────────────────────────────────────
marker!(RecentlyvisitedOpen, recently_id::CMD_OPEN, 0x1854a729, "ui.recentlyvisited.command_open");
marker!(RecentlyvisitedShow, recently_id::CMD_SHOW, 0x1854b2e1, "ui.recentlyvisited.command_show");
marker!(RecentlyvisitedPush, recently_id::CMD_PUSH, 0x1854b6c9, "ui.recentlyvisited.command_push");
marker!(RecentlyvisitedHide, recently_id::CMD_HIDE, 0x1854bab1, "ui.recentlyvisited.command_hide");
marker!(RecentlyvisitedClose, recently_id::CMD_CLOSE, 0x1854be99, "ui.recentlyvisited.command_close");
typed!(recentlyvisited::UpdateLabels, Command::RecentlyvisitedUpdateLabels, fprt_sys::ui::recentlyvisited::labels::Labels, recently_id::CMD_UPDATE_LABELS, 0x1854ab11, "ui.recentlyvisited.command_update_labels");
pooled!(recentlyvisited::UpdateAddresses, Command::RecentlyvisitedUpdateAddresses, fprt_sys::ui::AddressList, recently_id::CMD_UPDATE_ADDRESSES, 0x1854aef9, "ui.recentlyvisited.command_update_addresses");

// ─── blocked ──────────────────────────────────────────────────────────────────
marker!(BlockedOpen, blocked_id::CMD_OPEN, 0x18562dc9, "ui.blocked.command_open");
marker!(BlockedShow, blocked_id::CMD_SHOW, 0x18563981, "ui.blocked.command_show");
marker!(BlockedPush, blocked_id::CMD_PUSH, 0x18563d69, "ui.blocked.command_push");
marker!(BlockedHide, blocked_id::CMD_HIDE, 0x18564151, "ui.blocked.command_hide");
marker!(BlockedClose, blocked_id::CMD_CLOSE, 0x18564539, "ui.blocked.command_close");
typed!(blocked::UpdateLabels, Command::BlockedUpdateLabels, fprt_sys::ui::blocked::labels::Labels, blocked_id::CMD_UPDATE_LABELS, 0x185631b1, "ui.blocked.command_update_labels");
pooled!(blocked::UpdateAddresses, Command::BlockedUpdateAddresses, fprt_sys::ui::AddressList, blocked_id::CMD_UPDATE_ADDRESSES, 0x18563599, "ui.blocked.command_update_addresses");

// ─── zoom ─────────────────────────────────────────────────────────────────────
marker!(ZoomOpen, zoom_id::CMD_OPEN, 0x183622a9, "ui.zoom.command_open");
marker!(ZoomShow, zoom_id::CMD_SHOW, 0x18362a79, "ui.zoom.command_show");
marker!(ZoomPush, zoom_id::CMD_PUSH, 0x18362e61, "ui.zoom.command_push");
marker!(ZoomHide, zoom_id::CMD_HIDE, 0x18363249, "ui.zoom.command_hide");
marker!(ZoomClose, zoom_id::CMD_CLOSE, 0x18363631, "ui.zoom.command_close");
typed!(zoom::UpdateLabels, Command::ZoomUpdateLabels, fprt_sys::ui::zoom::labels::Labels, zoom_id::CMD_UPDATE_LABELS, 0x18362691, "ui.zoom.command_update_labels");

// ─── update ───────────────────────────────────────────────────────────────────
marker!(UpdateOpen, update_id::CMD_OPEN, 0x18bf76e9, "ui.update.command_open");
marker!(UpdateShow, update_id::CMD_SHOW, 0x18bf82a1, "ui.update.command_show");
marker!(UpdatePush, update_id::CMD_PUSH, 0x18bf8689, "ui.update.command_push");
marker!(UpdateHide, update_id::CMD_HIDE, 0x18bf8a71, "ui.update.command_hide");
marker!(UpdateClose, update_id::CMD_CLOSE, 0x18bf8e59, "ui.update.command_close");
typed!(update::UpdateLabels, Command::UpdateUpdateLabels, fprt_sys::ui::update::labels::Labels, update_id::CMD_UPDATE_LABELS, 0x18bf7ad1, "ui.update.command_update_labels");
typed!(update::UpdateData, Command::UpdateUpdateData, fprt_sys::ui::update::update_data::UpdateData, update_id::CMD_UPDATE_DATA, 0x18bf7eb9, "ui.update.command_update_data");

// ─── devtools ─────────────────────────────────────────────────────────────────
marker!(DevtoolsOpen, devtools_id::CMD_OPEN, 0x1891b029, "ui.devtools.command_open");
marker!(DevtoolsShow, devtools_id::CMD_SHOW, 0x1891bbe1, "ui.devtools.command_show");
marker!(DevtoolsPush, devtools_id::CMD_PUSH, 0x1891bfc9, "ui.devtools.command_push");
marker!(DevtoolsHide, devtools_id::CMD_HIDE, 0x1891c3b1, "ui.devtools.command_hide");
marker!(DevtoolsClose, devtools_id::CMD_CLOSE, 0x1891c799, "ui.devtools.command_close");
typed!(devtools::UpdateLabels, Command::DevtoolsUpdateLabels, fprt_sys::ui::devtools::labels::Labels, devtools_id::CMD_UPDATE_LABELS, 0x1891b411, "ui.devtools.command_update_labels");
pooled!(devtools::UpdateAddresses, Command::DevtoolsUpdateAddresses, fprt_sys::ui::AddressList, devtools_id::CMD_UPDATE_ADDRESSES, 0x18b03c79, "ui.devtools.command_update_addresses");

// ─── recovery ─────────────────────────────────────────────────────────────────
marker!(RecoveryOpen, recovery_id::CMD_OPEN, 0x18b034a9, "ui.recovery.command_open");
marker!(RecoveryShow, recovery_id::CMD_SHOW, 0x18b04061, "ui.recovery.command_show");
marker!(RecoveryHide, recovery_id::CMD_HIDE, 0x18b04449, "ui.recovery.command_hide");
marker!(RecoveryClose, recovery_id::CMD_CLOSE, 0x18b04831, "ui.recovery.command_close");
typed!(recovery::UpdateLabels, Command::RecoveryUpdateLabels, fprt_sys::ui::recovery::labels::Labels, recovery_id::CMD_UPDATE_LABELS, 0x18b03891, "ui.recovery.command_update_labels");
pooled!(recovery::UpdateAddresses, Command::RecoveryUpdateAddresses, fprt_sys::ui::AddressList, recovery_id::CMD_UPDATE_ADDRESSES, 0x18b03c79, "ui.recovery.command_update_addresses");

// ─── leaptofrogans ────────────────────────────────────────────────────────────
marker!(LeaptofrogansOpen, leaptofrogans_id::CMD_OPEN, 0x18732ba9, "ui.leaptofrogans.command_open");
marker!(LeaptofrogansShow, leaptofrogans_id::CMD_SHOW, 0x18733761, "ui.leaptofrogans.command_show");
marker!(LeaptofrogansPush, leaptofrogans_id::CMD_PUSH, 0x18733b49, "ui.leaptofrogans.command_push");
marker!(LeaptofrogansHide, leaptofrogans_id::CMD_HIDE, 0x18733f31, "ui.leaptofrogans.command_hide");
marker!(LeaptofrogansClose, leaptofrogans_id::CMD_CLOSE, 0x18734319, "ui.leaptofrogans.command_close");
typed!(leaptofrogans::UpdateLabels, Command::LeaptofrogansUpdateLabels, fprt_sys::ui::leaptofrogans::labels::Labels, leaptofrogans_id::CMD_UPDATE_LABELS, 0x18732f91, "ui.leaptofrogans.command_update_labels");
typed!(leaptofrogans::UpdateAddress, Command::LeaptofrogansUpdateAddress, fprt_sys::ui::leaptofrogans::update_address::UpdateAddress, leaptofrogans_id::CMD_UPDATE_ADDRESS, 0x18733379, "ui.leaptofrogans.command_update_address");

// ─── legalinformation ─────────────────────────────────────────────────────────
marker!(LegalinformationOpen, legal_id::CMD_OPEN, 0x18179e29, "ui.legalinformation.command_open");
marker!(LegalinformationShow, legal_id::CMD_SHOW, 0x1817a5f9, "ui.legalinformation.command_show");
marker!(LegalinformationPush, legal_id::CMD_PUSH, 0x1817a9e1, "ui.legalinformation.command_push");
marker!(LegalinformationHide, legal_id::CMD_HIDE, 0x1817adc9, "ui.legalinformation.command_hide");
marker!(LegalinformationClose, legal_id::CMD_CLOSE, 0x1817b1b1, "ui.legalinformation.command_close");
typed!(legalinformation::UpdateLabels, Command::LegalinformationUpdateLabels, fprt_sys::ui::legalinformation::labels::Labels, legal_id::CMD_UPDATE_LABELS, 0x1817a211, "ui.legalinformation.command_update_labels");
pooled!(legalinformation::UpdateLegalContent, Command::LegalinformationUpdateLegalContent, fprt_sys::ui::legalinformation::legal_content::UpdateLegalContent, legal_id::CMD_UPDATE_LEGAL_CONTENT, 0x1817b1b1, "ui.legalinformation.command_update_legal_content");

// ─── language ─────────────────────────────────────────────────────────────────
marker!(LanguageOpen, language_id::CMD_OPEN, 0x184564e9, "ui.language.command_open");
marker!(LanguageShow, language_id::CMD_SHOW, 0x184570a1, "ui.language.command_show");
marker!(LanguagePush, language_id::CMD_PUSH, 0x18457489, "ui.language.command_push");
marker!(LanguageHide, language_id::CMD_HIDE, 0x18457871, "ui.language.command_hide");
marker!(LanguageClose, language_id::CMD_CLOSE, 0x18457c59, "ui.language.command_close");
typed!(language::UpdateLabels, Command::LanguageUpdateLabels, fprt_sys::ui::language::labels::Labels, language_id::CMD_UPDATE_LABELS, 0x184568d1, "ui.language.command_update_labels");
pooled!(language::UpdateList, Command::LanguageUpdateList, fprt_sys::ui::language::update_list::UpdateList, language_id::CMD_UPDATE_LIST, 0x18456cb9, "ui.language.command_update_list");

// ─── inputfa ──────────────────────────────────────────────────────────────────
marker!(InputfaOpen, inputfa_id::CMD_OPEN, 0x1826e069, "ui.inputfa.command_open");
marker!(InputfaShow, inputfa_id::CMD_SHOW, 0x1826f3f1, "ui.inputfa.command_show");
marker!(InputfaPush, inputfa_id::CMD_PUSH, 0x1826f7d9, "ui.inputfa.command_push");
marker!(InputfaHide, inputfa_id::CMD_HIDE, 0x1826fbc1, "ui.inputfa.command_hide");
marker!(InputfaClose, inputfa_id::CMD_CLOSE, 0x1826ffa9, "ui.inputfa.command_close");
marker!(InputfaUpdateErrorClear, inputfa_id::CMD_UPDATE_ERROR_CLEAR, 0x1826f009, "ui.inputfa.command_update_error_clear");
typed!(inputfa::UpdateLabels, Command::InputfaUpdateLabels, fprt_sys::ui::inputfa::labels::Labels, inputfa_id::CMD_UPDATE_LABELS, 0x1826e451, "ui.inputfa.command_update_labels");
typed!(inputfa::UpdateAddress, Command::InputfaUpdateAddress, fprt_sys::ui::inputfa::update_address::UpdateAddress, inputfa_id::CMD_UPDATE_ADDRESS, 0x1826e839, "ui.inputfa.command_update_address");
typed!(inputfa::UpdateErrorRaise, Command::InputfaUpdateErrorRaise, fprt_sys::ui::inputfa::update_error_raise::UpdateErrorRaise, inputfa_id::CMD_UPDATE_ERROR_RAISE, 0x1826ec21, "ui.inputfa.command_update_error_raise");

// ─── inspector (lifecycle = id-carriers on InspectorId) ───────────────────────
id_carrier!(InspectorOpen, Command::InspectorOpen, fprt_sys::ui::inspector::head::Head, reference, inspector_id::CMD_OPEN, 0x18b034a9, "ui.inspector.command_open");
id_carrier!(InspectorShow, Command::InspectorShow, fprt_sys::ui::inspector::head::Head, reference, inspector_id::CMD_SHOW, 0x18b053e9, "ui.inspector.command_show");
id_carrier!(InspectorHide, Command::InspectorHide, fprt_sys::ui::inspector::head::Head, reference, inspector_id::CMD_HIDE, 0x18b05bb9, "ui.inspector.command_hide");
id_carrier!(InspectorPush, Command::InspectorPush, fprt_sys::ui::inspector::head::Head, reference, inspector_id::CMD_PUSH, 0x18b057d1, "ui.inspector.command_push");
id_carrier!(InspectorClose, Command::InspectorClose, fprt_sys::ui::inspector::head::Head, reference, inspector_id::CMD_CLOSE, 0x18b05fa1, "ui.inspector.command_close");
typed!(inspector::UpdateAddress, Command::InspectorUpdateAddress, fprt_sys::ui::inspector::update_address::UpdateAddress, inspector_id::CMD_UPDATE_ADDRESS, 0x18b03891, "ui.inspector.command_update_address");
typed!(inspector::UpdateStatus, Command::InspectorUpdateStatus, fprt_sys::ui::inspector::update_status::UpdateStatus, inspector_id::CMD_UPDATE_STATUS, 0x18b03c79, "ui.inspector.command_update_status");
typed!(inspector::UpdateLabels, Command::InspectorUpdateLabels, fprt_sys::ui::inspector::labels::Labels, inspector_id::CMD_UPDATE_LABELS, 0x18b04061, "ui.inspector.command_update_labels");
pooled!(inspector::UpdateStepsLabels, Command::InspectorUpdateStepsLabels, fprt_sys::ui::inspector::update_steps_labels::UpdateStepsLabels, inspector_id::CMD_UPDATE_STEPS_LABELS, 0x18b04449, "ui.inspector.command_update_steps_labels");
pooled!(inspector::UpdateContentLabels, Command::InspectorUpdateContentLabels, fprt_sys::ui::inspector::update_content_labels::UpdateContentLabels, inspector_id::CMD_UPDATE_CONTENT_LABELS, 0x18b04831, "ui.inspector.command_update_content_labels");
typed!(inspector::UpdateContentViewer, Command::InspectorUpdateContentViewer, fprt_sys::ui::inspector::update_content_viewer::UpdateContentViewer, inspector_id::CMD_UPDATE_CONTENT_VIEWER, 0x18b04c19, "ui.inspector.command_update_content_viewer");
typed!(inspector::UpdateSync, Command::InspectorUpdateSync, fprt_sys::ui::inspector::update_sync::UpdateSync, inspector_id::CMD_UPDATE_SYNC, 0x18b05001, "ui.inspector.command_update_sync");

// ─── pad ──────────────────────────────────────────────────────────────────────
marker!(PadOpen, pad_id::CMD_OPEN, 0x17f919a9, "ui.pad.command_open");
marker!(PadShow, pad_id::CMD_SHOW, 0x17f92949, "ui.pad.command_show");
marker!(PadHide, pad_id::CMD_HIDE, 0x17f92d31, "ui.pad.command_hide");
marker!(PadClose, pad_id::CMD_CLOSE, 0x17f93119, "ui.pad.command_close");
marker!(PadBeginAnimation, pad_id::CMD_BEGIN_ANIMATION, 0x17f92179, "ui.pad.command_begin_animation");
marker!(PadEndAnimation, pad_id::CMD_END_ANIMATION, 0x17f92561, "ui.pad.command_end_animation");
typed!(pad::UpdateLayout, Command::PadUpdateLayout, fprt_sys::ui::pad::update_layout::UpdateLayout, pad_id::CMD_UPDATE_LAYOUT, 0x17e9df39, "ui.pad.command_update_layout");

/// Map a queued command to the `0x2195xx` class id `get_next_command` reports, so
/// the host knows which `_pop` to call. Covers every emittable command; the five
/// still-deferred encodes (`UpdateImages`, menu/sitehandler `UpdateVisual`,
/// `UpdateList`, `UpdateLegalContent`) have no constructor yet, so they can't be
/// emitted and fall through to [`StatusName::FALLBACK`].
pub(crate) fn status_name(command: &Command) -> StatusName {
    match command {
        // application
        Command::ApplicationUpdateZoom(_) => application::UpdateZoom::ID,
        Command::ApplicationUpdateLayout(_) => application::UpdateLayout::ID,
        Command::ApplicationUpdateDirectionality(_) => application::UpdateDirectionality::ID,
        Command::ApplicationAddClipboardText(_) => application::AddClipboardText::ID,
        Command::ApplicationAddClipboardImage(_) => application::AddClipboardImage::ID,
        Command::ApplicationOpenDirectory(_) => application::OpenDirectory::ID,
        Command::ApplicationLaunchWayOut(_) => application::LaunchWayOut::ID,
        Command::ApplicationReinitializeDevelopersDirectory => {
            ApplicationReinitializeDevelopersDirectory::ID
        }
        Command::ApplicationStop => ApplicationStop::ID,
        // menu
        Command::MenuOpen => MenuOpen::ID,
        Command::MenuShow => MenuShow::ID,
        Command::MenuPush => MenuPush::ID,
        Command::MenuHide => MenuHide::ID,
        Command::MenuClose => MenuClose::ID,
        Command::MenuUpdateLayout(_) => menu::UpdateLayout::ID,
        // sitehandler
        Command::SitehandlerOpen(_) => SitehandlerOpen::ID,
        Command::SitehandlerShow(_) => SitehandlerShow::ID,
        Command::SitehandlerPush(_) => SitehandlerPush::ID,
        Command::SitehandlerHide(_) => SitehandlerHide::ID,
        Command::SitehandlerClose(_) => SitehandlerClose::ID,
        Command::SitehandlerBeginAnimationInprogress(_) => SitehandlerBeginAnimationInprogress::ID,
        Command::SitehandlerEndAnimationInprogress(_) => SitehandlerEndAnimationInprogress::ID,
        Command::SitehandlerUpdateLayout(_) => sitehandler::UpdateLayout::ID,
        // favorites
        Command::FavoritesOpen => FavoritesOpen::ID,
        Command::FavoritesShow => FavoritesShow::ID,
        Command::FavoritesPush => FavoritesPush::ID,
        Command::FavoritesHide => FavoritesHide::ID,
        Command::FavoritesClose => FavoritesClose::ID,
        Command::FavoritesUpdateLabels(_) => favorites::UpdateLabels::ID,
        Command::FavoritesUpdateAddresses(_) => favorites::UpdateAddresses::ID,
        // recentlyvisited
        Command::RecentlyvisitedOpen => RecentlyvisitedOpen::ID,
        Command::RecentlyvisitedShow => RecentlyvisitedShow::ID,
        Command::RecentlyvisitedPush => RecentlyvisitedPush::ID,
        Command::RecentlyvisitedHide => RecentlyvisitedHide::ID,
        Command::RecentlyvisitedClose => RecentlyvisitedClose::ID,
        Command::RecentlyvisitedUpdateLabels(_) => recentlyvisited::UpdateLabels::ID,
        Command::RecentlyvisitedUpdateAddresses(_) => recentlyvisited::UpdateAddresses::ID,
        // blocked
        Command::BlockedOpen => BlockedOpen::ID,
        Command::BlockedShow => BlockedShow::ID,
        Command::BlockedPush => BlockedPush::ID,
        Command::BlockedHide => BlockedHide::ID,
        Command::BlockedClose => BlockedClose::ID,
        Command::BlockedUpdateLabels(_) => blocked::UpdateLabels::ID,
        Command::BlockedUpdateAddresses(_) => blocked::UpdateAddresses::ID,
        // zoom
        Command::ZoomOpen => ZoomOpen::ID,
        Command::ZoomShow => ZoomShow::ID,
        Command::ZoomPush => ZoomPush::ID,
        Command::ZoomHide => ZoomHide::ID,
        Command::ZoomClose => ZoomClose::ID,
        Command::ZoomUpdateLabels(_) => zoom::UpdateLabels::ID,
        // update
        Command::UpdateOpen => UpdateOpen::ID,
        Command::UpdateShow => UpdateShow::ID,
        Command::UpdatePush => UpdatePush::ID,
        Command::UpdateHide => UpdateHide::ID,
        Command::UpdateClose => UpdateClose::ID,
        Command::UpdateUpdateLabels(_) => update::UpdateLabels::ID,
        Command::UpdateUpdateData(_) => update::UpdateData::ID,
        // devtools
        Command::DevtoolsOpen => DevtoolsOpen::ID,
        Command::DevtoolsShow => DevtoolsShow::ID,
        Command::DevtoolsPush => DevtoolsPush::ID,
        Command::DevtoolsHide => DevtoolsHide::ID,
        Command::DevtoolsClose => DevtoolsClose::ID,
        Command::DevtoolsUpdateLabels(_) => devtools::UpdateLabels::ID,
        Command::DevtoolsUpdateAddresses(_) => devtools::UpdateAddresses::ID,
        // recovery
        Command::RecoveryOpen => RecoveryOpen::ID,
        Command::RecoveryShow => RecoveryShow::ID,
        Command::RecoveryHide => RecoveryHide::ID,
        Command::RecoveryClose => RecoveryClose::ID,
        Command::RecoveryUpdateLabels(_) => recovery::UpdateLabels::ID,
        Command::RecoveryUpdateAddresses(_) => recovery::UpdateAddresses::ID,
        // leaptofrogans
        Command::LeaptofrogansOpen => LeaptofrogansOpen::ID,
        Command::LeaptofrogansShow => LeaptofrogansShow::ID,
        Command::LeaptofrogansPush => LeaptofrogansPush::ID,
        Command::LeaptofrogansHide => LeaptofrogansHide::ID,
        Command::LeaptofrogansClose => LeaptofrogansClose::ID,
        Command::LeaptofrogansUpdateLabels(_) => leaptofrogans::UpdateLabels::ID,
        Command::LeaptofrogansUpdateAddress(_) => leaptofrogans::UpdateAddress::ID,
        // legalinformation
        Command::LegalinformationOpen => LegalinformationOpen::ID,
        Command::LegalinformationShow => LegalinformationShow::ID,
        Command::LegalinformationPush => LegalinformationPush::ID,
        Command::LegalinformationHide => LegalinformationHide::ID,
        Command::LegalinformationClose => LegalinformationClose::ID,
        Command::LegalinformationUpdateLabels(_) => legalinformation::UpdateLabels::ID,
        // language
        Command::LanguageOpen => LanguageOpen::ID,
        Command::LanguageShow => LanguageShow::ID,
        Command::LanguagePush => LanguagePush::ID,
        Command::LanguageHide => LanguageHide::ID,
        Command::LanguageClose => LanguageClose::ID,
        Command::LanguageUpdateLabels(_) => language::UpdateLabels::ID,
        // inputfa
        Command::InputfaOpen => InputfaOpen::ID,
        Command::InputfaShow => InputfaShow::ID,
        Command::InputfaPush => InputfaPush::ID,
        Command::InputfaHide => InputfaHide::ID,
        Command::InputfaClose => InputfaClose::ID,
        Command::InputfaUpdateErrorClear => InputfaUpdateErrorClear::ID,
        Command::InputfaUpdateLabels(_) => inputfa::UpdateLabels::ID,
        Command::InputfaUpdateAddress(_) => inputfa::UpdateAddress::ID,
        Command::InputfaUpdateErrorRaise(_) => inputfa::UpdateErrorRaise::ID,
        // inspector
        Command::InspectorOpen(_) => InspectorOpen::ID,
        Command::InspectorShow(_) => InspectorShow::ID,
        Command::InspectorHide(_) => InspectorHide::ID,
        Command::InspectorPush(_) => InspectorPush::ID,
        Command::InspectorClose(_) => InspectorClose::ID,
        Command::InspectorUpdateAddress(_) => inspector::UpdateAddress::ID,
        Command::InspectorUpdateStatus(_) => inspector::UpdateStatus::ID,
        Command::InspectorUpdateLabels(_) => inspector::UpdateLabels::ID,
        Command::InspectorUpdateStepsLabels(_) => inspector::UpdateStepsLabels::ID,
        Command::InspectorUpdateContentLabels(_) => inspector::UpdateContentLabels::ID,
        Command::InspectorUpdateContentViewer(_) => inspector::UpdateContentViewer::ID,
        Command::InspectorUpdateSync(_) => inspector::UpdateSync::ID,
        // pad
        Command::PadOpen => PadOpen::ID,
        Command::PadShow => PadShow::ID,
        Command::PadHide => PadHide::ID,
        Command::PadClose => PadClose::ID,
        Command::PadBeginAnimation => PadBeginAnimation::ID,
        Command::PadEndAnimation => PadEndAnimation::ID,
        Command::PadUpdateLayout(_) => pad::UpdateLayout::ID,
        // complex / nested payloads (now fully wired)
        Command::ApplicationUpdateImages(_) => application::UpdateImages::ID,
        Command::MenuUpdateVisual(_) => menu::UpdateVisual::ID,
        Command::SitehandlerUpdateVisual(_) => sitehandler::UpdateVisual::ID,
        Command::LanguageUpdateList(_) => language::UpdateList::ID,
        Command::LegalinformationUpdateLegalContent(_) => {
            legalinformation::UpdateLegalContent::ID
        }
        // `Command` is `#[non_exhaustive]`, so a future variant lands here.
        _ => StatusName::FALLBACK,
    }
}
