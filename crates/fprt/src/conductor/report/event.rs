//! `impl Report for Event` — forward a decoded [`fprt_core::Event`] to the engine in
//! one call, by dispatching each variant to its typed report.
//!
//! Lets a proxy / forwarder do `conductor.report(elapsed, event)` instead of
//! matching the enum itself. Lives here (not in a downstream crate) because `Report`
//! is sealed and the orphan rule needs the trait local.
//!
//! Every variant forwards faithfully: the address-selection events carry a
//! [`Selection`](fprt_core::component::selection::Selection) (the entries) and the
//! inspector ref-events carry their [`InspectorId`](fprt_core::component::inspector::InspectorId),
//! so the re-sent event reproduces the original data exactly.

use fprt_core::Event;

use crate::conductor::Conductor;
use crate::conductor::component as c;
use crate::conductor::report::{Report, sealed};
use crate::error::EngineError;

impl sealed::Sealed for Event<'_> {}

impl Report for Event<'_> {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        match self {
            // ── payload-carrying events: delegate to the payload's own `send` ──
            Event::ApplicationStart(p) => p.send(conductor),
            Event::ApplicationLeaptofrogans(p) => p.send(conductor),
            Event::ApplicationMenuAccessWanted(p) => p.send(conductor),
            Event::ApplicationChangeLayout(p) => p.send(conductor),
            Event::InputfaChange(p) => p.send(conductor),
            Event::InputfaOk(p) => p.send(conductor),
            Event::InspectorChangeAutosync(p) => p.send(conductor),
            Event::InspectorContentSelected(p) => p.send(conductor),
            Event::InspectorStepSelected(p) => p.send(conductor),
            Event::LanguageOk(p) => p.send(conductor),
            Event::MenuButtonTriggered(p) => p.send(conductor),
            Event::SitehandlerButtonTriggered(p) => p.send(conductor),
            Event::SitehandlerForceClose(p) => p.send(conductor),
            Event::ZoomOk(p) => p.send(conductor),

            // ── dataless markers: construct the client's marker token ──
            Event::ApplicationTimeout => c::application::ReportTimeout::new().send(conductor),
            Event::ApplicationMenuAccessUnwanted => {
                c::application::ReportMenuAccessUnwanted::new().send(conductor)
            }
            Event::ApplicationQuit => c::application::ReportQuit::new().send(conductor),
            Event::BlockedRemoveAll => c::blocked::ReportRemoveAll::new().send(conductor),
            Event::BlockedCancel => c::blocked::ReportCancel::new().send(conductor),
            Event::DevtoolsCancel => c::devtools::ReportCancel::new().send(conductor),
            Event::FavoritesRemoveAll => c::favorites::ReportRemoveAll::new().send(conductor),
            Event::FavoritesCancel => c::favorites::ReportCancel::new().send(conductor),
            Event::InputfaCancel => c::inputfa::ReportCancel::new().send(conductor),
            Event::LanguageCancel => c::language::ReportCancel::new().send(conductor),
            Event::LeaptofrogansConfirm => c::leaptofrogans::ReportConfirm::new().send(conductor),
            Event::LeaptofrogansCancel => c::leaptofrogans::ReportCancel::new().send(conductor),
            Event::LeaptofrogansBlock => c::leaptofrogans::ReportBlock::new().send(conductor),
            Event::LeaptofrogansPurge => c::leaptofrogans::ReportPurge::new().send(conductor),
            Event::LeaptofrogansClose => c::leaptofrogans::ReportClose::new().send(conductor),
            Event::LegalinformationClose => {
                c::legalinformation::ReportClose::new().send(conductor)
            }
            Event::RecentlyvisitedDeleteAll => {
                c::recentlyvisited::ReportDeleteAll::new().send(conductor)
            }
            Event::RecentlyvisitedCancel => c::recentlyvisited::ReportCancel::new().send(conductor),
            Event::RecoveryCancel => c::recovery::ReportCancel::new().send(conductor),
            Event::UpdateCancel => c::update::ReportCancel::new().send(conductor),
            Event::ZoomCancel => c::zoom::ReportCancel::new().send(conductor),

            // ── address-selection events: forward the selected entries ──
            Event::FavoritesOpen(p) => c::favorites::ReportOpen::new(&p.addresses).send(conductor),
            Event::FavoritesRemove(p) => {
                c::favorites::ReportRemove::new(&p.addresses).send(conductor)
            }
            Event::RecentlyvisitedOpen(p) => {
                c::recentlyvisited::ReportOpen::new(&p.addresses).send(conductor)
            }
            Event::RecentlyvisitedDelete(p) => {
                c::recentlyvisited::ReportDelete::new(&p.addresses).send(conductor)
            }
            Event::BlockedRemove(p) => c::blocked::ReportRemove::new(&p.addresses).send(conductor),
            Event::DevtoolsInspect(p) => {
                c::devtools::ReportInspect::new(&p.addresses).send(conductor)
            }
            Event::RecoveryOpen(p) => c::recovery::ReportOpen::new(&p.addresses).send(conductor),

            // ── inspector ref-events: forward which window ──
            Event::InspectorSynchronize(id) => {
                c::inspector::ReportSynchronize::new(id).send(conductor)
            }
            Event::InspectorRerun(id) => c::inspector::ReportRerun::new(id).send(conductor),
            Event::InspectorClose(id) => c::inspector::ReportClose::new(id).send(conductor),

            // `Event` is `#[non_exhaustive]`; a future variant lands here unforwarded.
            _ => Ok(()),
        }
    }
}
