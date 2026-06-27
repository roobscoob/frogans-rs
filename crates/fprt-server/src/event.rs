//! Host→engine event transport: the [`EventDecode`] seam each `_report`
//! trampoline rides.
//!
//! The inbound mirror of [`command`](crate::command). Events are *borrowed* — the
//! decoded [`Event`] views the inbound payload for the call's duration — so
//! [`EventDecode::decode`] returns `Event<'_>` tied to the raw reference. The
//! generic `report::<E>` trampoline (in [`engine`]) decodes, then runs the engine
//! for one turn; `E` only supplies the raw type and the one-line decode.
//!
//! Two shapes: [`typed_event!`] (`from_raw` builds a payload-carrying variant) and
//! [`marker_event!`] (a no-data variant — the raw payload, whatever its type, is
//! ignored).
//!
//! [`engine`]: crate::engine

use fprt_core::Event;
use fprt_core::component::{
    application, inputfa, inspector, language, menu, selection, sitehandler, zoom,
};
use fprt_sys::ui::{AddressSelection, EventTag};

/// What a `_report` trampoline needs to turn an inbound raw payload into a borrowed
/// [`Event`]. One impl per event (or marker); `report::<E>` does the rest.
pub(crate) trait EventDecode {
    /// The `0x17`/`0x18` error-block base for this call (error-code DB §5.2), used
    /// for framework-level reject codes (`BASE + offset`, e.g. bad context).
    const BASE: u32;
    /// Human breadcrumb stamped onto a caught panic (`"ui.application.event_start"`).
    const OP: &'static str;
    /// The raw `#[repr(C)]` payload the host supplies.
    type Raw;
    /// Decode the inbound payload into a borrowed [`Event`] for this turn.
    fn decode(raw: &Self::Raw) -> Event<'_>;
}

/// Event whose `from_raw` builds a payload-carrying [`Event`] variant. `$from` is
/// the full `Payload::from_raw` path (passed whole so the macro never has to suffix
/// `::` onto a type fragment).
macro_rules! typed_event {
    ($name:ident, $variant:path, $from:path, $raw:ty, $base:expr, $op:expr) => {
        /// A payload-carrying event (host → engine).
        pub(crate) struct $name;

        impl EventDecode for $name {
            const BASE: u32 = $base;
            const OP: &'static str = $op;
            type Raw = $raw;

            fn decode(raw: &Self::Raw) -> Event<'_> {
                $variant($from(raw))
            }
        }
    };
}

/// No-data event marker: the raw payload (whatever its type) is ignored and the
/// unit [`Event`] variant returned.
macro_rules! marker_event {
    ($name:ident, $variant:expr, $raw:ty, $base:expr, $op:expr) => {
        /// A no-data event marker (host → engine).
        pub(crate) struct $name;

        impl EventDecode for $name {
            const BASE: u32 = $base;
            const OP: &'static str = $op;
            type Raw = $raw;

            fn decode(_raw: &Self::Raw) -> Event<'_> {
                $variant
            }
        }
    };
}

// ─── application ──────────────────────────────────────────────────────────────
typed_event!(ApplicationStart, Event::ApplicationStart, application::ReportStart::from_raw, fprt_sys::ui::application::event_start::EventStart, 0x17e854b1, "ui.application.event_start");
typed_event!(ApplicationMenuAccessWanted, Event::ApplicationMenuAccessWanted, application::ReportMenuAccessWanted::from_raw, fprt_sys::ui::application::event_menu_access_wanted::MenuAccessWanted, 0x17e85c81, "ui.application.event_menu_access_wanted");
typed_event!(ApplicationLeaptofrogans, Event::ApplicationLeaptofrogans, application::ReportLeaptofrogans::from_raw, fprt_sys::ui::application::event_leaptofrogans::EventLeaptofrogans, 0x17e86c21, "ui.application.event_leaptofrogans");
marker_event!(ApplicationTimeout, Event::ApplicationTimeout, EventTag, 0x17e85899, "ui.application.event_timeout");
marker_event!(ApplicationMenuAccessUnwanted, Event::ApplicationMenuAccessUnwanted, EventTag, 0x17e86069, "ui.application.event_menu_access_unwanted");
marker_event!(ApplicationQuit, Event::ApplicationQuit, EventTag, 0x17e873f1, "ui.application.event_quit");
typed_event!(ApplicationChangeLayout, Event::ApplicationChangeLayout, application::ReportChangeLayout::from_raw, fprt_sys::ui::application::event_change_layout::EventChangeLayout, 0x17e87009, "ui.application.event_change_layout");

// ─── menu / sitehandler ───────────────────────────────────────────────────────
typed_event!(MenuButtonTriggered, Event::MenuButtonTriggered, menu::ReportButtonTriggered::from_raw, fprt_sys::ui::menu::button_triggered::ButtonTriggered, 0x1806d549, "ui.menu.event_button_triggered");
typed_event!(SitehandlerButtonTriggered, Event::SitehandlerButtonTriggered, sitehandler::ReportButtonTriggered::from_raw, fprt_sys::ui::sitehandler::button_triggered::ButtonTriggered, 0x1880ef19, "ui.sitehandler.event_button_triggered");
typed_event!(SitehandlerForceClose, Event::SitehandlerForceClose, sitehandler::ReportForceClose::from_raw, fprt_sys::ui::sitehandler::force_close::ForceClose, 0x1880f301, "ui.sitehandler.event_force_close");

// ─── favorites ────────────────────────────────────────────────────────────────
typed_event!(FavoritesOpen, Event::FavoritesOpen, selection::Selection::from_raw, AddressSelection, 0x186262c9, "ui.favorites.event_open");
typed_event!(FavoritesRemove, Event::FavoritesRemove, selection::Selection::from_raw, AddressSelection, 0x186262c9, "ui.favorites.event_remove");
marker_event!(FavoritesRemoveAll, Event::FavoritesRemoveAll, EventTag, 0x186266b1, "ui.favorites.event_remove_all");
marker_event!(FavoritesCancel, Event::FavoritesCancel, EventTag, 0x186266b1, "ui.favorites.event_cancel");

// ─── recentlyvisited ──────────────────────────────────────────────────────────
typed_event!(RecentlyvisitedOpen, Event::RecentlyvisitedOpen, selection::Selection::from_raw, AddressSelection, 0x18532089, "ui.recentlyvisited.event_open");
typed_event!(RecentlyvisitedDelete, Event::RecentlyvisitedDelete, selection::Selection::from_raw, AddressSelection, 0x18532471, "ui.recentlyvisited.event_delete");
marker_event!(RecentlyvisitedDeleteAll, Event::RecentlyvisitedDeleteAll, EventTag, 0x18532859, "ui.recentlyvisited.event_delete_all");
marker_event!(RecentlyvisitedCancel, Event::RecentlyvisitedCancel, EventTag, 0x18532c41, "ui.recentlyvisited.event_cancel");

// ─── blocked ──────────────────────────────────────────────────────────────────
typed_event!(BlockedRemove, Event::BlockedRemove, selection::Selection::from_raw, AddressSelection, 0x18533029, "ui.blocked.event_remove");
marker_event!(BlockedRemoveAll, Event::BlockedRemoveAll, EventTag, 0x18533411, "ui.blocked.event_remove_all");
marker_event!(BlockedCancel, Event::BlockedCancel, EventTag, 0x185337f9, "ui.blocked.event_cancel");

// ─── devtools ─────────────────────────────────────────────────────────────────
typed_event!(DevtoolsInspect, Event::DevtoolsInspect, selection::Selection::from_raw, AddressSelection, 0x18902989, "ui.devtools.event_inspect");
marker_event!(DevtoolsCancel, Event::DevtoolsCancel, EventTag, 0x18902d71, "ui.devtools.event_cancel");

// ─── recovery ─────────────────────────────────────────────────────────────────
typed_event!(RecoveryOpen, Event::RecoveryOpen, selection::Selection::from_raw, AddressSelection, 0x189f6bc9, "ui.recovery.event_open");
marker_event!(RecoveryCancel, Event::RecoveryCancel, EventTag, 0x189f6fb1, "ui.recovery.event_cancel");

// ─── inputfa ──────────────────────────────────────────────────────────────────
typed_event!(InputfaChange, Event::InputfaChange, inputfa::ReportChange::from_raw, fprt_sys::ui::inputfa::field_text::FieldText, 0x182559c9, "ui.inputfa.event_change");
typed_event!(InputfaOk, Event::InputfaOk, inputfa::ReportOk::from_raw, fprt_sys::ui::inputfa::field_text::FieldText, 0x18255db1, "ui.inputfa.event_ok");
marker_event!(InputfaCancel, Event::InputfaCancel, EventTag, 0x18256199, "ui.inputfa.event_cancel");

// ─── zoom ─────────────────────────────────────────────────────────────────────
typed_event!(ZoomOk, Event::ZoomOk, zoom::ReportOk::from_raw, fprt_sys::ui::zoom::event_ok::EventOk, 0x18349c09, "ui.zoom.event_ok");
marker_event!(ZoomCancel, Event::ZoomCancel, EventTag, 0x18349ff1, "ui.zoom.event_cancel");

// ─── language ─────────────────────────────────────────────────────────────────
typed_event!(LanguageOk, Event::LanguageOk, language::ReportOk::from_raw, fprt_sys::ui::language::event_ok::LanguageOk, 0x1843de49, "ui.language.event_ok");
marker_event!(LanguageCancel, Event::LanguageCancel, EventTag, 0x1843e231, "ui.language.event_cancel");

// ─── leaptofrogans ────────────────────────────────────────────────────────────
marker_event!(LeaptofrogansConfirm, Event::LeaptofrogansConfirm, EventTag, 0x1871a8f1, "ui.leaptofrogans.event_confirm");
marker_event!(LeaptofrogansCancel, Event::LeaptofrogansCancel, EventTag, 0x1871a8f1, "ui.leaptofrogans.event_cancel");
marker_event!(LeaptofrogansBlock, Event::LeaptofrogansBlock, EventTag, 0x1871acd9, "ui.leaptofrogans.event_block");
marker_event!(LeaptofrogansPurge, Event::LeaptofrogansPurge, EventTag, 0x1871b0c1, "ui.leaptofrogans.event_purge");
marker_event!(LeaptofrogansClose, Event::LeaptofrogansClose, EventTag, 0x1871b0c1, "ui.leaptofrogans.event_close");

// ─── legalinformation ─────────────────────────────────────────────────────────
marker_event!(LegalinformationClose, Event::LegalinformationClose, EventTag, 0x18161b71, "ui.legalinformation.event_close");

// ─── inspector ────────────────────────────────────────────────────────────────
typed_event!(InspectorStepSelected, Event::InspectorStepSelected, inspector::ReportStepSelected::from_raw, fprt_sys::ui::inspector::step_selected::StepSelected, 0x18aeae09, "ui.inspector.event_step_selected");
typed_event!(InspectorContentSelected, Event::InspectorContentSelected, inspector::ReportContentSelected::from_raw, fprt_sys::ui::inspector::content_selected::ContentSelected, 0x18aeb1f1, "ui.inspector.event_content_selected");
typed_event!(InspectorChangeAutosync, Event::InspectorChangeAutosync, inspector::ReportChangeAutosync::from_raw, fprt_sys::ui::inspector::change_autosync::ChangeAutosync, 0x18aeb9c1, "ui.inspector.event_change_autosync");
typed_event!(InspectorSynchronize, Event::InspectorSynchronize, inspector::ref_event_id, fprt_sys::ui::inspector::ref_event::RefEvent, 0x18aeb5d9, "ui.inspector.event_synchronize");
typed_event!(InspectorRerun, Event::InspectorRerun, inspector::ref_event_id, fprt_sys::ui::inspector::ref_event::RefEvent, 0x18aebda9, "ui.inspector.event_rerun");
typed_event!(InspectorClose, Event::InspectorClose, inspector::ref_event_id, fprt_sys::ui::inspector::ref_event::RefEvent, 0x18aec191, "ui.inspector.event_close");

// ─── update ───────────────────────────────────────────────────────────────────
marker_event!(UpdateCancel, Event::UpdateCancel, EventTag, 0x18bdf049, "ui.update.event_cancel");
