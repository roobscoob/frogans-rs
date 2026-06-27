//! The flat [`Event`] enum — every host→engine event, the inbound mirror of
//! [`Command`](crate::Command).
//!
//! Events are *borrowed* (their payloads view the inbound buffer for the call's
//! duration), so the enum carries a lifetime `'a`. The server matches on it; the
//! client constructs them. Grows toward all inbound events.

use crate::component::{
    application, inputfa, inspector, language, menu, selection, sitehandler, zoom,
};

/// A host→engine event delivered during a turn.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event<'a> {
    /// `application` — bootstrap, carrying the process locale.
    ApplicationStart(application::ReportStart<'a>),
    /// `application` — the host wake timer fired (no payload).
    ApplicationTimeout,
    /// `application` — open a Frogans address from the pad.
    ApplicationLeaptofrogans(application::ReportLeaptofrogans<'a>),
    /// `application` — the UI wants the menu shown.
    ApplicationMenuAccessWanted(application::ReportMenuAccessWanted),
    /// `application` — the UI wants the menu dismissed (no payload).
    ApplicationMenuAccessUnwanted,
    /// `application` — a layout change report (window geometry / site layouts).
    ApplicationChangeLayout(application::ReportChangeLayout),
    /// `application` — the host requests shutdown (no payload).
    ApplicationQuit,

    /// `blocked` — the selected addresses to remove.
    BlockedRemove(selection::Selection<'a>),
    /// `blocked` — remove every blocked address (no payload).
    BlockedRemoveAll,
    /// `blocked` — dismiss the dialog (no payload).
    BlockedCancel,

    /// `devtools` — inspect the selected developer-directory entries.
    DevtoolsInspect(selection::Selection<'a>),
    /// `devtools` — dismiss the dialog (no payload).
    DevtoolsCancel,

    /// `favorites` — open the selected addresses.
    FavoritesOpen(selection::Selection<'a>),
    /// `favorites` — the selected addresses to remove.
    FavoritesRemove(selection::Selection<'a>),
    /// `favorites` — remove every favorite (no payload).
    FavoritesRemoveAll,
    /// `favorites` — dismiss the dialog (no payload).
    FavoritesCancel,

    /// `inputfa` — the field text changed.
    InputfaChange(inputfa::ReportChange<'a>),
    /// `inputfa` — the user confirmed the entered address.
    InputfaOk(inputfa::ReportOk<'a>),
    /// `inputfa` — dismiss the dialog (no payload).
    InputfaCancel,

    /// `inspector` — the auto-sync toggle changed in a window.
    InspectorChangeAutosync(inspector::ReportChangeAutosync),
    /// `inspector` — the content selector changed in a window.
    InspectorContentSelected(inspector::ReportContentSelected),
    /// `inspector` — the run-step combobox changed in a window.
    InspectorStepSelected(inspector::ReportStepSelected),
    /// `inspector` — a window asked to synchronize (carries which window).
    InspectorSynchronize(inspector::InspectorId),
    /// `inspector` — a window asked to rerun (carries which window).
    InspectorRerun(inspector::InspectorId),
    /// `inspector` — a window closed (carries which window).
    InspectorClose(inspector::InspectorId),

    /// `language` — the user confirmed a language selection.
    LanguageOk(language::ReportOk<'a>),
    /// `language` — dismiss the dialog (no payload).
    LanguageCancel,

    /// `leaptofrogans` — confirm the candidate address (no payload).
    LeaptofrogansConfirm,
    /// `leaptofrogans` — dismiss the dialog (no payload).
    LeaptofrogansCancel,
    /// `leaptofrogans` — block the candidate address (no payload).
    LeaptofrogansBlock,
    /// `leaptofrogans` — purge the candidate address (no payload).
    LeaptofrogansPurge,
    /// `leaptofrogans` — the dialog closed (no payload).
    LeaptofrogansClose,

    /// `legalinformation` — the panel closed (no payload).
    LegalinformationClose,

    /// `menu` — an interactive menu entry was triggered.
    MenuButtonTriggered(menu::ReportButtonTriggered<'a>),

    /// `recentlyvisited` — open the selected addresses.
    RecentlyvisitedOpen(selection::Selection<'a>),
    /// `recentlyvisited` — delete the selected addresses.
    RecentlyvisitedDelete(selection::Selection<'a>),
    /// `recentlyvisited` — delete every recently-visited address (no payload).
    RecentlyvisitedDeleteAll,
    /// `recentlyvisited` — dismiss the dialog (no payload).
    RecentlyvisitedCancel,

    /// `recovery` — open the selected recoverable addresses.
    RecoveryOpen(selection::Selection<'a>),
    /// `recovery` — dismiss the dialog (no payload).
    RecoveryCancel,

    /// `sitehandler` — an interactive zone in a site slide was triggered.
    SitehandlerButtonTriggered(sitehandler::ReportButtonTriggered<'a>),
    /// `sitehandler` — a site window was force-closed.
    SitehandlerForceClose(sitehandler::ReportForceClose),

    /// `update` — dismiss the dialog (no payload).
    UpdateCancel,

    /// `zoom` — the user confirmed a zoom level.
    ZoomOk(zoom::ReportOk),
    /// `zoom` — dismiss the dialog (no payload).
    ZoomCancel,
}
