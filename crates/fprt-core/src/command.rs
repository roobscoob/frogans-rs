//! The flat [`Command`] enum — every engine→host command, one variant each.
//!
//! Shared vocabulary: the client *receives* and matches these; the server
//! *produces* them. Flat by design (`Command::ApplicationUpdateZoom(_)`, one
//! `match` level); each variant's payload type lives in its [`component`] module.
//!
//! [`component`]: crate::component

use crate::component::{
    application, blocked, devtools, favorites, inputfa, inspector, language, leaptofrogans,
    legalinformation, menu, pad, recentlyvisited, recovery, sitehandler, update, zoom,
};
use crate::pool::OwnedPool;

impl Command {
    /// Deep-copy every pooled byte this command holds into `pool`, returning a
    /// command that borrows no other pool.
    ///
    /// The proxy uses this to re-emit a command it drained from *another* engine:
    /// the source's payload still points into the source's foreign mempool, which is
    /// freed when the source turn ends — re-emitting it by move would hand the host a
    /// dangling view. Copying the bytes into our own `pool` first makes the re-emit
    /// sound. Variants with no pooled bytes (markers, id-carriers, scalar payloads)
    /// pass through untouched.
    #[must_use]
    pub fn copy_into(self, pool: &OwnedPool) -> Command {
        match self {
            Command::ApplicationUpdateImages(c) => {
                Command::ApplicationUpdateImages(c.copy_into(pool))
            }
            Command::ApplicationAddClipboardText(c) => {
                Command::ApplicationAddClipboardText(c.copy_into(pool))
            }
            Command::ApplicationAddClipboardImage(c) => {
                Command::ApplicationAddClipboardImage(c.copy_into(pool))
            }
            Command::ApplicationLaunchWayOut(c) => {
                Command::ApplicationLaunchWayOut(c.copy_into(pool))
            }
            Command::FavoritesUpdateLabels(c) => Command::FavoritesUpdateLabels(c.copy_into(pool)),
            Command::FavoritesUpdateAddresses(c) => {
                Command::FavoritesUpdateAddresses(c.copy_into(pool))
            }
            Command::RecentlyvisitedUpdateLabels(c) => {
                Command::RecentlyvisitedUpdateLabels(c.copy_into(pool))
            }
            Command::RecentlyvisitedUpdateAddresses(c) => {
                Command::RecentlyvisitedUpdateAddresses(c.copy_into(pool))
            }
            Command::BlockedUpdateLabels(c) => Command::BlockedUpdateLabels(c.copy_into(pool)),
            Command::BlockedUpdateAddresses(c) => Command::BlockedUpdateAddresses(c.copy_into(pool)),
            Command::ZoomUpdateLabels(c) => Command::ZoomUpdateLabels(c.copy_into(pool)),
            Command::UpdateUpdateLabels(c) => Command::UpdateUpdateLabels(c.copy_into(pool)),
            Command::UpdateUpdateData(c) => Command::UpdateUpdateData(c.copy_into(pool)),
            Command::DevtoolsUpdateLabels(c) => Command::DevtoolsUpdateLabels(c.copy_into(pool)),
            Command::DevtoolsUpdateAddresses(c) => {
                Command::DevtoolsUpdateAddresses(c.copy_into(pool))
            }
            Command::RecoveryUpdateLabels(c) => Command::RecoveryUpdateLabels(c.copy_into(pool)),
            Command::RecoveryUpdateAddresses(c) => {
                Command::RecoveryUpdateAddresses(c.copy_into(pool))
            }
            Command::LeaptofrogansUpdateLabels(c) => {
                Command::LeaptofrogansUpdateLabels(c.copy_into(pool))
            }
            Command::LeaptofrogansUpdateAddress(c) => {
                Command::LeaptofrogansUpdateAddress(c.copy_into(pool))
            }
            Command::LegalinformationUpdateLabels(c) => {
                Command::LegalinformationUpdateLabels(c.copy_into(pool))
            }
            Command::LegalinformationUpdateLegalContent(c) => {
                Command::LegalinformationUpdateLegalContent(c.copy_into(pool))
            }
            Command::LanguageUpdateLabels(c) => Command::LanguageUpdateLabels(c.copy_into(pool)),
            Command::LanguageUpdateList(c) => Command::LanguageUpdateList(c.copy_into(pool)),
            Command::InputfaUpdateLabels(c) => Command::InputfaUpdateLabels(c.copy_into(pool)),
            Command::InputfaUpdateAddress(c) => Command::InputfaUpdateAddress(c.copy_into(pool)),
            Command::InputfaUpdateErrorRaise(c) => {
                Command::InputfaUpdateErrorRaise(c.copy_into(pool))
            }
            Command::InspectorUpdateAddress(c) => Command::InspectorUpdateAddress(c.copy_into(pool)),
            Command::InspectorUpdateLabels(c) => Command::InspectorUpdateLabels(c.copy_into(pool)),
            Command::InspectorUpdateStepsLabels(c) => {
                Command::InspectorUpdateStepsLabels(c.copy_into(pool))
            }
            Command::InspectorUpdateContentLabels(c) => {
                Command::InspectorUpdateContentLabels(c.copy_into(pool))
            }
            Command::InspectorUpdateContentViewer(c) => {
                Command::InspectorUpdateContentViewer(c.copy_into(pool))
            }
            Command::MenuUpdateVisual(c) => Command::MenuUpdateVisual(c.copy_into(pool)),
            Command::SitehandlerUpdateVisual(c) => {
                Command::SitehandlerUpdateVisual(c.copy_into(pool))
            }
            // No pooled bytes — markers, id-carriers, and scalar payloads (zoom,
            // layout, directionality, open-directory, status, sync) pass through.
            other => other,
        }
    }
}

/// A command the engine emitted during a turn (engine → host).
// The size spread (unit markers vs. `UpdateImages`' 16-slot tooltip array) is
// deliberate, not boxed: a `Command` is produced and consumed one at a time through
// a short per-turn outbox, never bulk-stored, so the per-variant size doesn't
// compound — and boxing the big variants would add an allocation on the emit path
// and ripple `Box::new`/deref through every match site in the client and server.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
#[non_exhaustive]
pub enum Command {
    /// `application` — the built-in UI image set (sent once at start).
    ApplicationUpdateImages(application::UpdateImages),
    /// `application` — current zoom level.
    ApplicationUpdateZoom(application::UpdateZoom),
    /// `application` — an application-level layout scalar (host-discarded).
    ApplicationUpdateLayout(application::UpdateLayout),
    /// `application` — the text-directionality enum.
    ApplicationUpdateDirectionality(application::UpdateDirectionality),
    /// `application` — text to place on the system clipboard.
    ApplicationAddClipboardText(application::AddClipboardText),
    /// `application` — image to place on the system clipboard.
    ApplicationAddClipboardImage(application::AddClipboardImage),
    /// `application` — reveal a known directory in the file manager.
    ApplicationOpenDirectory(application::OpenDirectory),
    /// `application` — re-initialize the developers directory.
    ApplicationReinitializeDevelopersDirectory,
    /// `application` — a URL to open externally.
    ApplicationLaunchWayOut(application::LaunchWayOut),
    /// `application` — the engine asks the host to stop.
    ApplicationStop,

    /// `favorites` — open the dialog.
    FavoritesOpen,
    /// `favorites` — show the dialog.
    FavoritesShow,
    /// `favorites` — push the dialog.
    FavoritesPush,
    /// `favorites` — hide the dialog.
    FavoritesHide,
    /// `favorites` — close the dialog.
    FavoritesClose,
    /// `favorites` — the dialog's localized strings.
    FavoritesUpdateLabels(favorites::UpdateLabels),
    /// `favorites` — the address list.
    FavoritesUpdateAddresses(favorites::UpdateAddresses),

    /// `recentlyvisited` — open the dialog.
    RecentlyvisitedOpen,
    /// `recentlyvisited` — show the dialog.
    RecentlyvisitedShow,
    /// `recentlyvisited` — push the dialog.
    RecentlyvisitedPush,
    /// `recentlyvisited` — hide the dialog.
    RecentlyvisitedHide,
    /// `recentlyvisited` — close the dialog.
    RecentlyvisitedClose,
    /// `recentlyvisited` — the dialog's localized strings.
    RecentlyvisitedUpdateLabels(recentlyvisited::UpdateLabels),
    /// `recentlyvisited` — the address list.
    RecentlyvisitedUpdateAddresses(recentlyvisited::UpdateAddresses),

    /// `blocked` — open the dialog.
    BlockedOpen,
    /// `blocked` — show the dialog.
    BlockedShow,
    /// `blocked` — push the dialog.
    BlockedPush,
    /// `blocked` — hide the dialog.
    BlockedHide,
    /// `blocked` — close the dialog.
    BlockedClose,
    /// `blocked` — the dialog's localized strings.
    BlockedUpdateLabels(blocked::UpdateLabels),
    /// `blocked` — the address list.
    BlockedUpdateAddresses(blocked::UpdateAddresses),

    /// `zoom` — open the dialog.
    ZoomOpen,
    /// `zoom` — show the dialog.
    ZoomShow,
    /// `zoom` — push the dialog.
    ZoomPush,
    /// `zoom` — hide the dialog.
    ZoomHide,
    /// `zoom` — close the dialog.
    ZoomClose,
    /// `zoom` — the dialog's localized strings.
    ZoomUpdateLabels(zoom::UpdateLabels),

    /// `update` — open the dialog.
    UpdateOpen,
    /// `update` — show the dialog.
    UpdateShow,
    /// `update` — push the dialog.
    UpdatePush,
    /// `update` — hide the dialog.
    UpdateHide,
    /// `update` — close the dialog.
    UpdateClose,
    /// `update` — the dialog's localized strings.
    UpdateUpdateLabels(update::UpdateLabels),
    /// `update` — the dialog's two URIs.
    UpdateUpdateData(update::UpdateData),

    /// `devtools` — open the dialog.
    DevtoolsOpen,
    /// `devtools` — show the dialog.
    DevtoolsShow,
    /// `devtools` — push the dialog.
    DevtoolsPush,
    /// `devtools` — hide the dialog.
    DevtoolsHide,
    /// `devtools` — close the dialog.
    DevtoolsClose,
    /// `devtools` — the dialog's localized strings.
    DevtoolsUpdateLabels(devtools::UpdateLabels),
    /// `devtools` — the developer-directory list.
    DevtoolsUpdateAddresses(devtools::UpdateAddresses),

    /// `recovery` — open the dialog.
    RecoveryOpen,
    /// `recovery` — show the dialog.
    RecoveryShow,
    /// `recovery` — hide the dialog.
    RecoveryHide,
    /// `recovery` — close the dialog.
    RecoveryClose,
    /// `recovery` — the dialog's localized strings.
    RecoveryUpdateLabels(recovery::UpdateLabels),
    /// `recovery` — the recoverable-address list.
    RecoveryUpdateAddresses(recovery::UpdateAddresses),

    /// `leaptofrogans` — open the dialog.
    LeaptofrogansOpen,
    /// `leaptofrogans` — show the dialog.
    LeaptofrogansShow,
    /// `leaptofrogans` — push the dialog.
    LeaptofrogansPush,
    /// `leaptofrogans` — hide the dialog.
    LeaptofrogansHide,
    /// `leaptofrogans` — close the dialog.
    LeaptofrogansClose,
    /// `leaptofrogans` — the dialog's localized strings.
    LeaptofrogansUpdateLabels(leaptofrogans::UpdateLabels),
    /// `leaptofrogans` — the candidate address + compliance.
    LeaptofrogansUpdateAddress(leaptofrogans::UpdateAddress),

    /// `legalinformation` — open the panel.
    LegalinformationOpen,
    /// `legalinformation` — show the panel.
    LegalinformationShow,
    /// `legalinformation` — push the panel.
    LegalinformationPush,
    /// `legalinformation` — hide the panel.
    LegalinformationHide,
    /// `legalinformation` — close the panel.
    LegalinformationClose,
    /// `legalinformation` — the panel's localized strings.
    LegalinformationUpdateLabels(legalinformation::UpdateLabels),
    /// `legalinformation` — the nested legal-document content tree.
    LegalinformationUpdateLegalContent(legalinformation::UpdateLegalContent),

    /// `language` — open the dialog.
    LanguageOpen,
    /// `language` — show the dialog.
    LanguageShow,
    /// `language` — push the dialog.
    LanguagePush,
    /// `language` — hide the dialog.
    LanguageHide,
    /// `language` — close the dialog.
    LanguageClose,
    /// `language` — the dialog's localized strings.
    LanguageUpdateLabels(language::UpdateLabels),
    /// `language` — the selectable-language list.
    LanguageUpdateList(language::UpdateList),

    /// `inputfa` — open the dialog.
    InputfaOpen,
    /// `inputfa` — show the dialog.
    InputfaShow,
    /// `inputfa` — push the dialog.
    InputfaPush,
    /// `inputfa` — hide the dialog.
    InputfaHide,
    /// `inputfa` — close the dialog.
    InputfaClose,
    /// `inputfa` — clear the inline error.
    InputfaUpdateErrorClear,
    /// `inputfa` — the dialog's localized strings.
    InputfaUpdateLabels(inputfa::UpdateLabels),
    /// `inputfa` — the canonical address for the field.
    InputfaUpdateAddress(inputfa::UpdateAddress),
    /// `inputfa` — inline error text to display.
    InputfaUpdateErrorRaise(inputfa::UpdateErrorRaise),

    /// `inspector` — open a window. Carries the target window id.
    InspectorOpen(inspector::InspectorId),
    /// `inspector` — show a window.
    InspectorShow(inspector::InspectorId),
    /// `inspector` — hide a window.
    InspectorHide(inspector::InspectorId),
    /// `inspector` — push a window.
    InspectorPush(inspector::InspectorId),
    /// `inspector` — close a window.
    InspectorClose(inspector::InspectorId),
    /// `inspector` — the address shown in a window.
    InspectorUpdateAddress(inspector::UpdateAddress),
    /// `inspector` — a window's run status.
    InspectorUpdateStatus(inspector::UpdateStatus),
    /// `inspector` — a window's localized strings.
    InspectorUpdateLabels(inspector::UpdateLabels),
    /// `inspector` — a window's run-step combobox.
    InspectorUpdateStepsLabels(inspector::UpdateStepsLabels),
    /// `inspector` — a window's content selector.
    InspectorUpdateContentLabels(inspector::UpdateContentLabels),
    /// `inspector` — a document loaded into a window's viewer.
    InspectorUpdateContentViewer(inspector::UpdateContentViewer),
    /// `inspector` — a window's auto-sync state.
    InspectorUpdateSync(inspector::UpdateSync),

    /// `pad` — open the pad window.
    PadOpen,
    /// `pad` — show the pad window.
    PadShow,
    /// `pad` — hide the pad window.
    PadHide,
    /// `pad` — close the pad window.
    PadClose,
    /// `pad` — begin the open/close animation.
    PadBeginAnimation,
    /// `pad` — end the open/close animation.
    PadEndAnimation,
    /// `pad` — the pad window's geometry.
    PadUpdateLayout(pad::UpdateLayout),

    /// `menu` — open the menu.
    MenuOpen,
    /// `menu` — show the menu.
    MenuShow,
    /// `menu` — push the menu.
    MenuPush,
    /// `menu` — hide the menu.
    MenuHide,
    /// `menu` — close the menu.
    MenuClose,
    /// `menu` — the rendered menu + interactive entries.
    MenuUpdateVisual(menu::UpdateVisual),
    /// `menu` — the menu's geometry (host-discarded).
    MenuUpdateLayout(menu::UpdateLayout),

    /// `sitehandler` — open a site window. Carries the target site id.
    SitehandlerOpen(sitehandler::SiteId),
    /// `sitehandler` — show a site window.
    SitehandlerShow(sitehandler::SiteId),
    /// `sitehandler` — push a site window.
    SitehandlerPush(sitehandler::SiteId),
    /// `sitehandler` — hide a site window.
    SitehandlerHide(sitehandler::SiteId),
    /// `sitehandler` — close a site window.
    SitehandlerClose(sitehandler::SiteId),
    /// `sitehandler` — begin the in-progress animation at a site window.
    SitehandlerBeginAnimationInprogress(sitehandler::SiteId),
    /// `sitehandler` — end the in-progress animation at a site window.
    SitehandlerEndAnimationInprogress(sitehandler::SiteId),
    /// `sitehandler` — re-position / zoom a site window.
    SitehandlerUpdateLayout(sitehandler::UpdateLayout),
    /// `sitehandler` — the rendered site slides + interactive zones.
    SitehandlerUpdateVisual(sitehandler::UpdateVisual),
}

// ─── emit vocabulary: `From<payload> for Command` + marker tokens ────────────
//
// What powers `out.command(...)` on the server: a data payload converts via its
// `From` impl; a no-data command is a public marker token. Grows toward all
// commands (the bulk, mirroring the enum above).

/// A no-data command marker — `out.command(MenuOpen)`.
#[derive(Clone, Copy, Debug)]
pub struct MenuOpen;

impl From<MenuOpen> for Command {
    fn from(_: MenuOpen) -> Self {
        Command::MenuOpen
    }
}

impl From<application::UpdateZoom> for Command {
    fn from(p: application::UpdateZoom) -> Self {
        Command::ApplicationUpdateZoom(p)
    }
}

impl From<application::AddClipboardText> for Command {
    fn from(p: application::AddClipboardText) -> Self {
        Command::ApplicationAddClipboardText(p)
    }
}

// ─── application ──────────────────────────────────────────────────────────────

impl From<application::UpdateImages> for Command {
    fn from(p: application::UpdateImages) -> Self {
        Command::ApplicationUpdateImages(p)
    }
}

impl From<application::UpdateLayout> for Command {
    fn from(p: application::UpdateLayout) -> Self {
        Command::ApplicationUpdateLayout(p)
    }
}

impl From<application::UpdateDirectionality> for Command {
    fn from(p: application::UpdateDirectionality) -> Self {
        Command::ApplicationUpdateDirectionality(p)
    }
}

impl From<application::AddClipboardImage> for Command {
    fn from(p: application::AddClipboardImage) -> Self {
        Command::ApplicationAddClipboardImage(p)
    }
}

impl From<application::OpenDirectory> for Command {
    fn from(p: application::OpenDirectory) -> Self {
        Command::ApplicationOpenDirectory(p)
    }
}

impl From<application::LaunchWayOut> for Command {
    fn from(p: application::LaunchWayOut) -> Self {
        Command::ApplicationLaunchWayOut(p)
    }
}

/// Re-initialize the developers directory — `out.command(ApplicationReinitializeDevelopersDirectory)`.
#[derive(Clone, Copy, Debug)]
pub struct ApplicationReinitializeDevelopersDirectory;

impl From<ApplicationReinitializeDevelopersDirectory> for Command {
    fn from(_: ApplicationReinitializeDevelopersDirectory) -> Self {
        Command::ApplicationReinitializeDevelopersDirectory
    }
}

/// Ask the host to stop — `out.command(ApplicationStop)`.
#[derive(Clone, Copy, Debug)]
pub struct ApplicationStop;

impl From<ApplicationStop> for Command {
    fn from(_: ApplicationStop) -> Self {
        Command::ApplicationStop
    }
}

// ─── favorites ────────────────────────────────────────────────────────────────

/// Open the favorites dialog.
#[derive(Clone, Copy, Debug)]
pub struct FavoritesOpen;

impl From<FavoritesOpen> for Command {
    fn from(_: FavoritesOpen) -> Self {
        Command::FavoritesOpen
    }
}

/// Show the favorites dialog.
#[derive(Clone, Copy, Debug)]
pub struct FavoritesShow;

impl From<FavoritesShow> for Command {
    fn from(_: FavoritesShow) -> Self {
        Command::FavoritesShow
    }
}

/// Push the favorites dialog.
#[derive(Clone, Copy, Debug)]
pub struct FavoritesPush;

impl From<FavoritesPush> for Command {
    fn from(_: FavoritesPush) -> Self {
        Command::FavoritesPush
    }
}

/// Hide the favorites dialog.
#[derive(Clone, Copy, Debug)]
pub struct FavoritesHide;

impl From<FavoritesHide> for Command {
    fn from(_: FavoritesHide) -> Self {
        Command::FavoritesHide
    }
}

/// Close the favorites dialog.
#[derive(Clone, Copy, Debug)]
pub struct FavoritesClose;

impl From<FavoritesClose> for Command {
    fn from(_: FavoritesClose) -> Self {
        Command::FavoritesClose
    }
}

impl From<favorites::UpdateLabels> for Command {
    fn from(p: favorites::UpdateLabels) -> Self {
        Command::FavoritesUpdateLabels(p)
    }
}

impl From<favorites::UpdateAddresses> for Command {
    fn from(p: favorites::UpdateAddresses) -> Self {
        Command::FavoritesUpdateAddresses(p)
    }
}

// ─── recentlyvisited ──────────────────────────────────────────────────────────

/// Open the recently-visited dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecentlyvisitedOpen;

impl From<RecentlyvisitedOpen> for Command {
    fn from(_: RecentlyvisitedOpen) -> Self {
        Command::RecentlyvisitedOpen
    }
}

/// Show the recently-visited dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecentlyvisitedShow;

impl From<RecentlyvisitedShow> for Command {
    fn from(_: RecentlyvisitedShow) -> Self {
        Command::RecentlyvisitedShow
    }
}

/// Push the recently-visited dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecentlyvisitedPush;

impl From<RecentlyvisitedPush> for Command {
    fn from(_: RecentlyvisitedPush) -> Self {
        Command::RecentlyvisitedPush
    }
}

/// Hide the recently-visited dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecentlyvisitedHide;

impl From<RecentlyvisitedHide> for Command {
    fn from(_: RecentlyvisitedHide) -> Self {
        Command::RecentlyvisitedHide
    }
}

/// Close the recently-visited dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecentlyvisitedClose;

impl From<RecentlyvisitedClose> for Command {
    fn from(_: RecentlyvisitedClose) -> Self {
        Command::RecentlyvisitedClose
    }
}

impl From<recentlyvisited::UpdateLabels> for Command {
    fn from(p: recentlyvisited::UpdateLabels) -> Self {
        Command::RecentlyvisitedUpdateLabels(p)
    }
}

impl From<recentlyvisited::UpdateAddresses> for Command {
    fn from(p: recentlyvisited::UpdateAddresses) -> Self {
        Command::RecentlyvisitedUpdateAddresses(p)
    }
}

// ─── blocked ──────────────────────────────────────────────────────────────────

/// Open the blocked dialog.
#[derive(Clone, Copy, Debug)]
pub struct BlockedOpen;

impl From<BlockedOpen> for Command {
    fn from(_: BlockedOpen) -> Self {
        Command::BlockedOpen
    }
}

/// Show the blocked dialog.
#[derive(Clone, Copy, Debug)]
pub struct BlockedShow;

impl From<BlockedShow> for Command {
    fn from(_: BlockedShow) -> Self {
        Command::BlockedShow
    }
}

/// Push the blocked dialog.
#[derive(Clone, Copy, Debug)]
pub struct BlockedPush;

impl From<BlockedPush> for Command {
    fn from(_: BlockedPush) -> Self {
        Command::BlockedPush
    }
}

/// Hide the blocked dialog.
#[derive(Clone, Copy, Debug)]
pub struct BlockedHide;

impl From<BlockedHide> for Command {
    fn from(_: BlockedHide) -> Self {
        Command::BlockedHide
    }
}

/// Close the blocked dialog.
#[derive(Clone, Copy, Debug)]
pub struct BlockedClose;

impl From<BlockedClose> for Command {
    fn from(_: BlockedClose) -> Self {
        Command::BlockedClose
    }
}

impl From<blocked::UpdateLabels> for Command {
    fn from(p: blocked::UpdateLabels) -> Self {
        Command::BlockedUpdateLabels(p)
    }
}

impl From<blocked::UpdateAddresses> for Command {
    fn from(p: blocked::UpdateAddresses) -> Self {
        Command::BlockedUpdateAddresses(p)
    }
}

// ─── zoom ─────────────────────────────────────────────────────────────────────

/// Open the zoom dialog.
#[derive(Clone, Copy, Debug)]
pub struct ZoomOpen;

impl From<ZoomOpen> for Command {
    fn from(_: ZoomOpen) -> Self {
        Command::ZoomOpen
    }
}

/// Show the zoom dialog.
#[derive(Clone, Copy, Debug)]
pub struct ZoomShow;

impl From<ZoomShow> for Command {
    fn from(_: ZoomShow) -> Self {
        Command::ZoomShow
    }
}

/// Push the zoom dialog.
#[derive(Clone, Copy, Debug)]
pub struct ZoomPush;

impl From<ZoomPush> for Command {
    fn from(_: ZoomPush) -> Self {
        Command::ZoomPush
    }
}

/// Hide the zoom dialog.
#[derive(Clone, Copy, Debug)]
pub struct ZoomHide;

impl From<ZoomHide> for Command {
    fn from(_: ZoomHide) -> Self {
        Command::ZoomHide
    }
}

/// Close the zoom dialog.
#[derive(Clone, Copy, Debug)]
pub struct ZoomClose;

impl From<ZoomClose> for Command {
    fn from(_: ZoomClose) -> Self {
        Command::ZoomClose
    }
}

impl From<zoom::UpdateLabels> for Command {
    fn from(p: zoom::UpdateLabels) -> Self {
        Command::ZoomUpdateLabels(p)
    }
}

// ─── update ───────────────────────────────────────────────────────────────────

/// Open the update dialog.
#[derive(Clone, Copy, Debug)]
pub struct UpdateOpen;

impl From<UpdateOpen> for Command {
    fn from(_: UpdateOpen) -> Self {
        Command::UpdateOpen
    }
}

/// Show the update dialog.
#[derive(Clone, Copy, Debug)]
pub struct UpdateShow;

impl From<UpdateShow> for Command {
    fn from(_: UpdateShow) -> Self {
        Command::UpdateShow
    }
}

/// Push the update dialog.
#[derive(Clone, Copy, Debug)]
pub struct UpdatePush;

impl From<UpdatePush> for Command {
    fn from(_: UpdatePush) -> Self {
        Command::UpdatePush
    }
}

/// Hide the update dialog.
#[derive(Clone, Copy, Debug)]
pub struct UpdateHide;

impl From<UpdateHide> for Command {
    fn from(_: UpdateHide) -> Self {
        Command::UpdateHide
    }
}

/// Close the update dialog.
#[derive(Clone, Copy, Debug)]
pub struct UpdateClose;

impl From<UpdateClose> for Command {
    fn from(_: UpdateClose) -> Self {
        Command::UpdateClose
    }
}

impl From<update::UpdateLabels> for Command {
    fn from(p: update::UpdateLabels) -> Self {
        Command::UpdateUpdateLabels(p)
    }
}

impl From<update::UpdateData> for Command {
    fn from(p: update::UpdateData) -> Self {
        Command::UpdateUpdateData(p)
    }
}

// ─── devtools ─────────────────────────────────────────────────────────────────

/// Open the devtools dialog.
#[derive(Clone, Copy, Debug)]
pub struct DevtoolsOpen;

impl From<DevtoolsOpen> for Command {
    fn from(_: DevtoolsOpen) -> Self {
        Command::DevtoolsOpen
    }
}

/// Show the devtools dialog.
#[derive(Clone, Copy, Debug)]
pub struct DevtoolsShow;

impl From<DevtoolsShow> for Command {
    fn from(_: DevtoolsShow) -> Self {
        Command::DevtoolsShow
    }
}

/// Push the devtools dialog.
#[derive(Clone, Copy, Debug)]
pub struct DevtoolsPush;

impl From<DevtoolsPush> for Command {
    fn from(_: DevtoolsPush) -> Self {
        Command::DevtoolsPush
    }
}

/// Hide the devtools dialog.
#[derive(Clone, Copy, Debug)]
pub struct DevtoolsHide;

impl From<DevtoolsHide> for Command {
    fn from(_: DevtoolsHide) -> Self {
        Command::DevtoolsHide
    }
}

/// Close the devtools dialog.
#[derive(Clone, Copy, Debug)]
pub struct DevtoolsClose;

impl From<DevtoolsClose> for Command {
    fn from(_: DevtoolsClose) -> Self {
        Command::DevtoolsClose
    }
}

impl From<devtools::UpdateLabels> for Command {
    fn from(p: devtools::UpdateLabels) -> Self {
        Command::DevtoolsUpdateLabels(p)
    }
}

impl From<devtools::UpdateAddresses> for Command {
    fn from(p: devtools::UpdateAddresses) -> Self {
        Command::DevtoolsUpdateAddresses(p)
    }
}

// ─── recovery ─────────────────────────────────────────────────────────────────

/// Open the recovery dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryOpen;

impl From<RecoveryOpen> for Command {
    fn from(_: RecoveryOpen) -> Self {
        Command::RecoveryOpen
    }
}

/// Show the recovery dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryShow;

impl From<RecoveryShow> for Command {
    fn from(_: RecoveryShow) -> Self {
        Command::RecoveryShow
    }
}

/// Hide the recovery dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryHide;

impl From<RecoveryHide> for Command {
    fn from(_: RecoveryHide) -> Self {
        Command::RecoveryHide
    }
}

/// Close the recovery dialog.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryClose;

impl From<RecoveryClose> for Command {
    fn from(_: RecoveryClose) -> Self {
        Command::RecoveryClose
    }
}

impl From<recovery::UpdateLabels> for Command {
    fn from(p: recovery::UpdateLabels) -> Self {
        Command::RecoveryUpdateLabels(p)
    }
}

impl From<recovery::UpdateAddresses> for Command {
    fn from(p: recovery::UpdateAddresses) -> Self {
        Command::RecoveryUpdateAddresses(p)
    }
}

// ─── leaptofrogans ────────────────────────────────────────────────────────────

/// Open the leaptofrogans dialog.
#[derive(Clone, Copy, Debug)]
pub struct LeaptofrogansOpen;

impl From<LeaptofrogansOpen> for Command {
    fn from(_: LeaptofrogansOpen) -> Self {
        Command::LeaptofrogansOpen
    }
}

/// Show the leaptofrogans dialog.
#[derive(Clone, Copy, Debug)]
pub struct LeaptofrogansShow;

impl From<LeaptofrogansShow> for Command {
    fn from(_: LeaptofrogansShow) -> Self {
        Command::LeaptofrogansShow
    }
}

/// Push the leaptofrogans dialog.
#[derive(Clone, Copy, Debug)]
pub struct LeaptofrogansPush;

impl From<LeaptofrogansPush> for Command {
    fn from(_: LeaptofrogansPush) -> Self {
        Command::LeaptofrogansPush
    }
}

/// Hide the leaptofrogans dialog.
#[derive(Clone, Copy, Debug)]
pub struct LeaptofrogansHide;

impl From<LeaptofrogansHide> for Command {
    fn from(_: LeaptofrogansHide) -> Self {
        Command::LeaptofrogansHide
    }
}

/// Close the leaptofrogans dialog.
#[derive(Clone, Copy, Debug)]
pub struct LeaptofrogansClose;

impl From<LeaptofrogansClose> for Command {
    fn from(_: LeaptofrogansClose) -> Self {
        Command::LeaptofrogansClose
    }
}

impl From<leaptofrogans::UpdateLabels> for Command {
    fn from(p: leaptofrogans::UpdateLabels) -> Self {
        Command::LeaptofrogansUpdateLabels(p)
    }
}

impl From<leaptofrogans::UpdateAddress> for Command {
    fn from(p: leaptofrogans::UpdateAddress) -> Self {
        Command::LeaptofrogansUpdateAddress(p)
    }
}

// ─── legalinformation ─────────────────────────────────────────────────────────

/// Open the legalinformation panel.
#[derive(Clone, Copy, Debug)]
pub struct LegalinformationOpen;

impl From<LegalinformationOpen> for Command {
    fn from(_: LegalinformationOpen) -> Self {
        Command::LegalinformationOpen
    }
}

/// Show the legalinformation panel.
#[derive(Clone, Copy, Debug)]
pub struct LegalinformationShow;

impl From<LegalinformationShow> for Command {
    fn from(_: LegalinformationShow) -> Self {
        Command::LegalinformationShow
    }
}

/// Push the legalinformation panel.
#[derive(Clone, Copy, Debug)]
pub struct LegalinformationPush;

impl From<LegalinformationPush> for Command {
    fn from(_: LegalinformationPush) -> Self {
        Command::LegalinformationPush
    }
}

/// Hide the legalinformation panel.
#[derive(Clone, Copy, Debug)]
pub struct LegalinformationHide;

impl From<LegalinformationHide> for Command {
    fn from(_: LegalinformationHide) -> Self {
        Command::LegalinformationHide
    }
}

/// Close the legalinformation panel.
#[derive(Clone, Copy, Debug)]
pub struct LegalinformationClose;

impl From<LegalinformationClose> for Command {
    fn from(_: LegalinformationClose) -> Self {
        Command::LegalinformationClose
    }
}

impl From<legalinformation::UpdateLabels> for Command {
    fn from(p: legalinformation::UpdateLabels) -> Self {
        Command::LegalinformationUpdateLabels(p)
    }
}

impl From<legalinformation::UpdateLegalContent> for Command {
    fn from(p: legalinformation::UpdateLegalContent) -> Self {
        Command::LegalinformationUpdateLegalContent(p)
    }
}

// ─── language ─────────────────────────────────────────────────────────────────

/// Open the language dialog.
#[derive(Clone, Copy, Debug)]
pub struct LanguageOpen;

impl From<LanguageOpen> for Command {
    fn from(_: LanguageOpen) -> Self {
        Command::LanguageOpen
    }
}

/// Show the language dialog.
#[derive(Clone, Copy, Debug)]
pub struct LanguageShow;

impl From<LanguageShow> for Command {
    fn from(_: LanguageShow) -> Self {
        Command::LanguageShow
    }
}

/// Push the language dialog.
#[derive(Clone, Copy, Debug)]
pub struct LanguagePush;

impl From<LanguagePush> for Command {
    fn from(_: LanguagePush) -> Self {
        Command::LanguagePush
    }
}

/// Hide the language dialog.
#[derive(Clone, Copy, Debug)]
pub struct LanguageHide;

impl From<LanguageHide> for Command {
    fn from(_: LanguageHide) -> Self {
        Command::LanguageHide
    }
}

/// Close the language dialog.
#[derive(Clone, Copy, Debug)]
pub struct LanguageClose;

impl From<LanguageClose> for Command {
    fn from(_: LanguageClose) -> Self {
        Command::LanguageClose
    }
}

impl From<language::UpdateLabels> for Command {
    fn from(p: language::UpdateLabels) -> Self {
        Command::LanguageUpdateLabels(p)
    }
}

impl From<language::UpdateList> for Command {
    fn from(p: language::UpdateList) -> Self {
        Command::LanguageUpdateList(p)
    }
}

// ─── inputfa ──────────────────────────────────────────────────────────────────

/// Open the inputfa dialog.
#[derive(Clone, Copy, Debug)]
pub struct InputfaOpen;

impl From<InputfaOpen> for Command {
    fn from(_: InputfaOpen) -> Self {
        Command::InputfaOpen
    }
}

/// Show the inputfa dialog.
#[derive(Clone, Copy, Debug)]
pub struct InputfaShow;

impl From<InputfaShow> for Command {
    fn from(_: InputfaShow) -> Self {
        Command::InputfaShow
    }
}

/// Push the inputfa dialog.
#[derive(Clone, Copy, Debug)]
pub struct InputfaPush;

impl From<InputfaPush> for Command {
    fn from(_: InputfaPush) -> Self {
        Command::InputfaPush
    }
}

/// Hide the inputfa dialog.
#[derive(Clone, Copy, Debug)]
pub struct InputfaHide;

impl From<InputfaHide> for Command {
    fn from(_: InputfaHide) -> Self {
        Command::InputfaHide
    }
}

/// Close the inputfa dialog.
#[derive(Clone, Copy, Debug)]
pub struct InputfaClose;

impl From<InputfaClose> for Command {
    fn from(_: InputfaClose) -> Self {
        Command::InputfaClose
    }
}

/// Clear the inputfa inline error.
#[derive(Clone, Copy, Debug)]
pub struct InputfaUpdateErrorClear;

impl From<InputfaUpdateErrorClear> for Command {
    fn from(_: InputfaUpdateErrorClear) -> Self {
        Command::InputfaUpdateErrorClear
    }
}

impl From<inputfa::UpdateLabels> for Command {
    fn from(p: inputfa::UpdateLabels) -> Self {
        Command::InputfaUpdateLabels(p)
    }
}

impl From<inputfa::UpdateAddress> for Command {
    fn from(p: inputfa::UpdateAddress) -> Self {
        Command::InputfaUpdateAddress(p)
    }
}

impl From<inputfa::UpdateErrorRaise> for Command {
    fn from(p: inputfa::UpdateErrorRaise) -> Self {
        Command::InputfaUpdateErrorRaise(p)
    }
}

// ─── inspector (id-carrying lifecycle + data) ─────────────────────────────────

/// Open an inspector window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct InspectorOpen(pub inspector::InspectorId);

impl From<InspectorOpen> for Command {
    fn from(t: InspectorOpen) -> Self {
        Command::InspectorOpen(t.0)
    }
}

/// Show an inspector window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct InspectorShow(pub inspector::InspectorId);

impl From<InspectorShow> for Command {
    fn from(t: InspectorShow) -> Self {
        Command::InspectorShow(t.0)
    }
}

/// Hide an inspector window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct InspectorHide(pub inspector::InspectorId);

impl From<InspectorHide> for Command {
    fn from(t: InspectorHide) -> Self {
        Command::InspectorHide(t.0)
    }
}

/// Push an inspector window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct InspectorPush(pub inspector::InspectorId);

impl From<InspectorPush> for Command {
    fn from(t: InspectorPush) -> Self {
        Command::InspectorPush(t.0)
    }
}

/// Close an inspector window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct InspectorClose(pub inspector::InspectorId);

impl From<InspectorClose> for Command {
    fn from(t: InspectorClose) -> Self {
        Command::InspectorClose(t.0)
    }
}

impl From<inspector::UpdateAddress> for Command {
    fn from(p: inspector::UpdateAddress) -> Self {
        Command::InspectorUpdateAddress(p)
    }
}

impl From<inspector::UpdateStatus> for Command {
    fn from(p: inspector::UpdateStatus) -> Self {
        Command::InspectorUpdateStatus(p)
    }
}

impl From<inspector::UpdateLabels> for Command {
    fn from(p: inspector::UpdateLabels) -> Self {
        Command::InspectorUpdateLabels(p)
    }
}

impl From<inspector::UpdateStepsLabels> for Command {
    fn from(p: inspector::UpdateStepsLabels) -> Self {
        Command::InspectorUpdateStepsLabels(p)
    }
}

impl From<inspector::UpdateContentLabels> for Command {
    fn from(p: inspector::UpdateContentLabels) -> Self {
        Command::InspectorUpdateContentLabels(p)
    }
}

impl From<inspector::UpdateContentViewer> for Command {
    fn from(p: inspector::UpdateContentViewer) -> Self {
        Command::InspectorUpdateContentViewer(p)
    }
}

impl From<inspector::UpdateSync> for Command {
    fn from(p: inspector::UpdateSync) -> Self {
        Command::InspectorUpdateSync(p)
    }
}

// ─── pad ──────────────────────────────────────────────────────────────────────

/// Open the pad window.
#[derive(Clone, Copy, Debug)]
pub struct PadOpen;

impl From<PadOpen> for Command {
    fn from(_: PadOpen) -> Self {
        Command::PadOpen
    }
}

/// Show the pad window.
#[derive(Clone, Copy, Debug)]
pub struct PadShow;

impl From<PadShow> for Command {
    fn from(_: PadShow) -> Self {
        Command::PadShow
    }
}

/// Hide the pad window.
#[derive(Clone, Copy, Debug)]
pub struct PadHide;

impl From<PadHide> for Command {
    fn from(_: PadHide) -> Self {
        Command::PadHide
    }
}

/// Close the pad window.
#[derive(Clone, Copy, Debug)]
pub struct PadClose;

impl From<PadClose> for Command {
    fn from(_: PadClose) -> Self {
        Command::PadClose
    }
}

/// Begin the pad open/close animation.
#[derive(Clone, Copy, Debug)]
pub struct PadBeginAnimation;

impl From<PadBeginAnimation> for Command {
    fn from(_: PadBeginAnimation) -> Self {
        Command::PadBeginAnimation
    }
}

/// End the pad open/close animation.
#[derive(Clone, Copy, Debug)]
pub struct PadEndAnimation;

impl From<PadEndAnimation> for Command {
    fn from(_: PadEndAnimation) -> Self {
        Command::PadEndAnimation
    }
}

impl From<pad::UpdateLayout> for Command {
    fn from(p: pad::UpdateLayout) -> Self {
        Command::PadUpdateLayout(p)
    }
}

// ─── menu ─────────────────────────────────────────────────────────────────────

/// Show the menu.
#[derive(Clone, Copy, Debug)]
pub struct MenuShow;

impl From<MenuShow> for Command {
    fn from(_: MenuShow) -> Self {
        Command::MenuShow
    }
}

/// Push the menu.
#[derive(Clone, Copy, Debug)]
pub struct MenuPush;

impl From<MenuPush> for Command {
    fn from(_: MenuPush) -> Self {
        Command::MenuPush
    }
}

/// Hide the menu.
#[derive(Clone, Copy, Debug)]
pub struct MenuHide;

impl From<MenuHide> for Command {
    fn from(_: MenuHide) -> Self {
        Command::MenuHide
    }
}

/// Close the menu.
#[derive(Clone, Copy, Debug)]
pub struct MenuClose;

impl From<MenuClose> for Command {
    fn from(_: MenuClose) -> Self {
        Command::MenuClose
    }
}

impl From<menu::UpdateVisual> for Command {
    fn from(p: menu::UpdateVisual) -> Self {
        Command::MenuUpdateVisual(p)
    }
}

impl From<menu::UpdateLayout> for Command {
    fn from(p: menu::UpdateLayout) -> Self {
        Command::MenuUpdateLayout(p)
    }
}

// ─── sitehandler (id-carrying lifecycle + data) ───────────────────────────────

/// Open a site window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerOpen(pub sitehandler::SiteId);

impl From<SitehandlerOpen> for Command {
    fn from(t: SitehandlerOpen) -> Self {
        Command::SitehandlerOpen(t.0)
    }
}

/// Show a site window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerShow(pub sitehandler::SiteId);

impl From<SitehandlerShow> for Command {
    fn from(t: SitehandlerShow) -> Self {
        Command::SitehandlerShow(t.0)
    }
}

/// Push a site window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerPush(pub sitehandler::SiteId);

impl From<SitehandlerPush> for Command {
    fn from(t: SitehandlerPush) -> Self {
        Command::SitehandlerPush(t.0)
    }
}

/// Hide a site window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerHide(pub sitehandler::SiteId);

impl From<SitehandlerHide> for Command {
    fn from(t: SitehandlerHide) -> Self {
        Command::SitehandlerHide(t.0)
    }
}

/// Close a site window at the given id.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerClose(pub sitehandler::SiteId);

impl From<SitehandlerClose> for Command {
    fn from(t: SitehandlerClose) -> Self {
        Command::SitehandlerClose(t.0)
    }
}

/// Begin the in-progress animation at a site window.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerBeginAnimationInprogress(pub sitehandler::SiteId);

impl From<SitehandlerBeginAnimationInprogress> for Command {
    fn from(t: SitehandlerBeginAnimationInprogress) -> Self {
        Command::SitehandlerBeginAnimationInprogress(t.0)
    }
}

/// End the in-progress animation at a site window.
#[derive(Clone, Copy, Debug)]
pub struct SitehandlerEndAnimationInprogress(pub sitehandler::SiteId);

impl From<SitehandlerEndAnimationInprogress> for Command {
    fn from(t: SitehandlerEndAnimationInprogress) -> Self {
        Command::SitehandlerEndAnimationInprogress(t.0)
    }
}

impl From<sitehandler::UpdateLayout> for Command {
    fn from(p: sitehandler::UpdateLayout) -> Self {
        Command::SitehandlerUpdateLayout(p)
    }
}

impl From<sitehandler::UpdateVisual> for Command {
    fn from(p: sitehandler::UpdateVisual) -> Self {
        Command::SitehandlerUpdateVisual(p)
    }
}
