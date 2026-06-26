//! Engine→host commands: the flat [`Command`] enum, the [`Commands`] iterator,
//! and the `command_id → *_pop → safe payload` dispatch.

use core::iter::FusedIterator;
use core::mem::MaybeUninit;
use std::sync::Arc;

use fprt_sys::Fprt;
use fprt_sys::ctx::Ctx;
use fprt_sys::mem::MempoolHandle;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::ustring::Ustring;

use crate::call::invoke;
use crate::engine::EngineInner;
use crate::error::EngineError;
use crate::pool::Pool;

use super::component::{
    application, blocked, devtools, favorites, inputfa, inspector, language, leaptofrogans,
    legalinformation, menu, pad, recentlyvisited, recovery, sitehandler, update, zoom,
};

/// A command the engine emitted during a turn (engine → host).
///
/// Flat by design: `Command::ApplicationUpdateZoom(_)`, not
/// `Command::Application(application::Command::UpdateZoom(_))` — one `match` level.
/// The payload type lives in its component module ([`application::UpdateZoom`]).
/// Grows toward all 16 components / 167 commands.
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

/// Why reading the command stream stopped.
#[derive(Debug)]
pub enum CommandError {
    /// An engine call (`get_next_command` or a payload pop) failed.
    Engine(EngineError),
    /// The engine emitted a command this wrapper doesn't model yet.
    ///
    /// It has no typed reader, so it can't be popped — and an un-popped command
    /// stays at the head of the queue forever (the next `get_next_command` just
    /// returns it again). So the stream stops here rather than spin on it.
    Unknown(StatusName),
}

impl From<EngineError> for CommandError {
    fn from(error: EngineError) -> Self {
        CommandError::Engine(error)
    }
}

impl core::fmt::Display for CommandError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CommandError::Engine(e) => write!(f, "{e}"),
            CommandError::Unknown(id) => write!(f, "unmodeled command {id:?}"),
        }
    }
}

impl std::error::Error for CommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CommandError::Engine(e) => Some(e),
            CommandError::Unknown(_) => None,
        }
    }
}

/// The engine→host command stream for one turn. Yields each queued command,
/// already converted to a safe [`Command`]; iteration ends when the queue drains.
///
/// An error is **terminal**: it is yielded once, then iteration stops. So a loop
/// that logs-and-continues (rather than `?`-propagating) can't spin on a
/// persistent error.
pub struct Commands<'a> {
    ctx: Ctx,
    engine: &'a Arc<EngineInner>,
    done: bool,
}

impl<'a> Commands<'a> {
    pub(crate) fn new(ctx: Ctx, engine: &'a Arc<EngineInner>) -> Self {
        Commands {
            ctx,
            engine,
            done: false,
        }
    }
}

impl Iterator for Commands<'_> {
    type Item = Result<Command, CommandError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let item = match next_command(self.ctx, self.engine) {
            Err(e) => Err(CommandError::Engine(e)),
            Ok(None) => {
                self.done = true; // queue drained → iteration ends
                return None;
            }
            Ok(Some(id)) => pop_command(self.ctx, self.engine, id),
        };
        if item.is_err() {
            self.done = true; // an error halts the iterator
        }
        Some(item)
    }
}

impl FusedIterator for Commands<'_> {}

fn next_command(ctx: Ctx, engine: &Arc<EngineInner>) -> Result<Option<StatusName>, EngineError> {
    let mut has = 0u32;
    let mut id = 0u32;
    // SAFETY: valid ctx + out pointers; table valid via the live engine.
    invoke(engine, |s, e, p| unsafe {
        (engine.methods().conductor_get_next_command)(ctx, &mut has, &mut id, s, e, p)
    })?;
    Ok((has != 0).then_some(StatusName(id)))
}

/// Everything one command needs to be decoded — implemented in the command's own
/// file (e.g. `component/application/cmd_update_zoom.rs`), so dispatch logic is
/// fully distributed. The central [`COMMANDS`] table just lists the types.
pub(crate) trait CommandPayload: Sized {
    /// The engine's `0x2195xx` command-class id.
    const ID: StatusName;
    /// The raw `#[repr(C)]` payload the engine fills.
    type Raw;
    /// The matching `*_pop` export.
    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw>;
    /// Convert the filled raw payload (+ its pool) into the safe struct.
    fn from_raw(raw: Self::Raw, pool: &Pool) -> Self;
    /// Wrap the safe struct in its [`Command`] variant.
    fn into_command(self) -> Command;
}

/// Define a no-payload command marker (a `Pop<StatusName>` lifecycle signal):
/// `marker_command!(Open, fprt_sys::ui::menu::CMD_OPEN, menu_open, MenuOpen);`
/// generates the unit type + its [`CommandPayload`] impl. Still add the matching
/// `Command::MenuOpen` variant and a `COMMANDS` row.
macro_rules! marker_command {
    ($name:ident, $id:expr, $export:ident, $variant:ident) => {
        /// A no-payload command marker (engine → host).
        pub(crate) struct $name;

        impl $crate::conductor::command::CommandPayload for $name {
            const ID: ::fprt_sys::ui::StatusName = $id;
            type Raw = ::fprt_sys::ui::StatusName;

            fn pop_fn(methods: &::fprt_sys::Fprt) -> ::fprt_sys::ui::Pop<Self::Raw> {
                methods.$export
            }

            fn from_raw(_raw: ::fprt_sys::ui::StatusName, _pool: &$crate::pool::Pool) -> Self {
                $name
            }

            fn into_command(self) -> $crate::Command {
                $crate::Command::$variant
            }
        }
    };
}
pub(crate) use marker_command;

/// A registry row's decoder: pop one command of type `C` and box it as a
/// [`Command`]. Uniform across all commands, so each row is `pop_typed::<C>`.
fn pop_typed<C: CommandPayload>(
    ctx: Ctx,
    engine: &Arc<EngineInner>,
) -> Result<Command, CommandError> {
    let pop_fn = C::pop_fn(engine.methods());
    // SAFETY: `pop_fn` is the export for this command; `ctx` + out pointers valid.
    let (raw, pool) = pop(engine, |out, s, e, p| unsafe { pop_fn(ctx, out, s, e, p) })?;
    Ok(C::from_raw(raw, &pool).into_command())
}

type Dispatch = fn(Ctx, &Arc<EngineInner>) -> Result<Command, CommandError>;

/// The command registry: one row per modeled command. The only central list that
/// grows as commands are added (alongside the matching `Command` variant).
static COMMANDS: &[(StatusName, Dispatch)] = &[
    (
        application::UpdateImages::ID,
        pop_typed::<application::UpdateImages>,
    ),
    (
        application::UpdateZoom::ID,
        pop_typed::<application::UpdateZoom>,
    ),
    (
        application::UpdateLayout::ID,
        pop_typed::<application::UpdateLayout>,
    ),
    (
        application::UpdateDirectionality::ID,
        pop_typed::<application::UpdateDirectionality>,
    ),
    (
        application::AddClipboardText::ID,
        pop_typed::<application::AddClipboardText>,
    ),
    (
        application::AddClipboardImage::ID,
        pop_typed::<application::AddClipboardImage>,
    ),
    (
        application::OpenDirectory::ID,
        pop_typed::<application::OpenDirectory>,
    ),
    (
        application::ReinitializeDevelopersDirectory::ID,
        pop_typed::<application::ReinitializeDevelopersDirectory>,
    ),
    (
        application::LaunchWayOut::ID,
        pop_typed::<application::LaunchWayOut>,
    ),
    (application::Stop::ID, pop_typed::<application::Stop>),
    (favorites::Open::ID, pop_typed::<favorites::Open>),
    (favorites::Show::ID, pop_typed::<favorites::Show>),
    (favorites::Push::ID, pop_typed::<favorites::Push>),
    (favorites::Hide::ID, pop_typed::<favorites::Hide>),
    (favorites::Close::ID, pop_typed::<favorites::Close>),
    (
        favorites::UpdateLabels::ID,
        pop_typed::<favorites::UpdateLabels>,
    ),
    (
        favorites::UpdateAddresses::ID,
        pop_typed::<favorites::UpdateAddresses>,
    ),
    (
        recentlyvisited::Open::ID,
        pop_typed::<recentlyvisited::Open>,
    ),
    (
        recentlyvisited::Show::ID,
        pop_typed::<recentlyvisited::Show>,
    ),
    (
        recentlyvisited::Push::ID,
        pop_typed::<recentlyvisited::Push>,
    ),
    (
        recentlyvisited::Hide::ID,
        pop_typed::<recentlyvisited::Hide>,
    ),
    (
        recentlyvisited::Close::ID,
        pop_typed::<recentlyvisited::Close>,
    ),
    (
        recentlyvisited::UpdateLabels::ID,
        pop_typed::<recentlyvisited::UpdateLabels>,
    ),
    (
        recentlyvisited::UpdateAddresses::ID,
        pop_typed::<recentlyvisited::UpdateAddresses>,
    ),
    (blocked::Open::ID, pop_typed::<blocked::Open>),
    (blocked::Show::ID, pop_typed::<blocked::Show>),
    (blocked::Push::ID, pop_typed::<blocked::Push>),
    (blocked::Hide::ID, pop_typed::<blocked::Hide>),
    (blocked::Close::ID, pop_typed::<blocked::Close>),
    (
        blocked::UpdateLabels::ID,
        pop_typed::<blocked::UpdateLabels>,
    ),
    (
        blocked::UpdateAddresses::ID,
        pop_typed::<blocked::UpdateAddresses>,
    ),
    (zoom::Open::ID, pop_typed::<zoom::Open>),
    (zoom::Show::ID, pop_typed::<zoom::Show>),
    (zoom::Push::ID, pop_typed::<zoom::Push>),
    (zoom::Hide::ID, pop_typed::<zoom::Hide>),
    (zoom::Close::ID, pop_typed::<zoom::Close>),
    (zoom::UpdateLabels::ID, pop_typed::<zoom::UpdateLabels>),
    (update::Open::ID, pop_typed::<update::Open>),
    (update::Show::ID, pop_typed::<update::Show>),
    (update::Push::ID, pop_typed::<update::Push>),
    (update::Hide::ID, pop_typed::<update::Hide>),
    (update::Close::ID, pop_typed::<update::Close>),
    (
        update::UpdateLabels::ID,
        pop_typed::<update::UpdateLabels>,
    ),
    (update::UpdateData::ID, pop_typed::<update::UpdateData>),
    (devtools::Open::ID, pop_typed::<devtools::Open>),
    (devtools::Show::ID, pop_typed::<devtools::Show>),
    (devtools::Push::ID, pop_typed::<devtools::Push>),
    (devtools::Hide::ID, pop_typed::<devtools::Hide>),
    (devtools::Close::ID, pop_typed::<devtools::Close>),
    (
        devtools::UpdateLabels::ID,
        pop_typed::<devtools::UpdateLabels>,
    ),
    (
        devtools::UpdateAddresses::ID,
        pop_typed::<devtools::UpdateAddresses>,
    ),
    (recovery::Open::ID, pop_typed::<recovery::Open>),
    (recovery::Show::ID, pop_typed::<recovery::Show>),
    (recovery::Hide::ID, pop_typed::<recovery::Hide>),
    (recovery::Close::ID, pop_typed::<recovery::Close>),
    (
        recovery::UpdateLabels::ID,
        pop_typed::<recovery::UpdateLabels>,
    ),
    (
        recovery::UpdateAddresses::ID,
        pop_typed::<recovery::UpdateAddresses>,
    ),
    (
        leaptofrogans::Open::ID,
        pop_typed::<leaptofrogans::Open>,
    ),
    (
        leaptofrogans::Show::ID,
        pop_typed::<leaptofrogans::Show>,
    ),
    (
        leaptofrogans::Push::ID,
        pop_typed::<leaptofrogans::Push>,
    ),
    (
        leaptofrogans::Hide::ID,
        pop_typed::<leaptofrogans::Hide>,
    ),
    (
        leaptofrogans::Close::ID,
        pop_typed::<leaptofrogans::Close>,
    ),
    (
        leaptofrogans::UpdateLabels::ID,
        pop_typed::<leaptofrogans::UpdateLabels>,
    ),
    (
        leaptofrogans::UpdateAddress::ID,
        pop_typed::<leaptofrogans::UpdateAddress>,
    ),
    (
        legalinformation::Open::ID,
        pop_typed::<legalinformation::Open>,
    ),
    (
        legalinformation::Show::ID,
        pop_typed::<legalinformation::Show>,
    ),
    (
        legalinformation::Push::ID,
        pop_typed::<legalinformation::Push>,
    ),
    (
        legalinformation::Hide::ID,
        pop_typed::<legalinformation::Hide>,
    ),
    (
        legalinformation::Close::ID,
        pop_typed::<legalinformation::Close>,
    ),
    (
        legalinformation::UpdateLabels::ID,
        pop_typed::<legalinformation::UpdateLabels>,
    ),
    (
        legalinformation::UpdateLegalContent::ID,
        pop_typed::<legalinformation::UpdateLegalContent>,
    ),
    (language::Open::ID, pop_typed::<language::Open>),
    (language::Show::ID, pop_typed::<language::Show>),
    (language::Push::ID, pop_typed::<language::Push>),
    (language::Hide::ID, pop_typed::<language::Hide>),
    (language::Close::ID, pop_typed::<language::Close>),
    (
        language::UpdateLabels::ID,
        pop_typed::<language::UpdateLabels>,
    ),
    (
        language::UpdateList::ID,
        pop_typed::<language::UpdateList>,
    ),
    (inputfa::Open::ID, pop_typed::<inputfa::Open>),
    (inputfa::Show::ID, pop_typed::<inputfa::Show>),
    (inputfa::Push::ID, pop_typed::<inputfa::Push>),
    (inputfa::Hide::ID, pop_typed::<inputfa::Hide>),
    (inputfa::Close::ID, pop_typed::<inputfa::Close>),
    (
        inputfa::UpdateErrorClear::ID,
        pop_typed::<inputfa::UpdateErrorClear>,
    ),
    (
        inputfa::UpdateLabels::ID,
        pop_typed::<inputfa::UpdateLabels>,
    ),
    (
        inputfa::UpdateAddress::ID,
        pop_typed::<inputfa::UpdateAddress>,
    ),
    (
        inputfa::UpdateErrorRaise::ID,
        pop_typed::<inputfa::UpdateErrorRaise>,
    ),
    (inspector::Open::ID, pop_typed::<inspector::Open>),
    (inspector::Show::ID, pop_typed::<inspector::Show>),
    (inspector::Hide::ID, pop_typed::<inspector::Hide>),
    (inspector::Push::ID, pop_typed::<inspector::Push>),
    (inspector::Close::ID, pop_typed::<inspector::Close>),
    (
        inspector::UpdateAddress::ID,
        pop_typed::<inspector::UpdateAddress>,
    ),
    (
        inspector::UpdateStatus::ID,
        pop_typed::<inspector::UpdateStatus>,
    ),
    (
        inspector::UpdateLabels::ID,
        pop_typed::<inspector::UpdateLabels>,
    ),
    (
        inspector::UpdateStepsLabels::ID,
        pop_typed::<inspector::UpdateStepsLabels>,
    ),
    (
        inspector::UpdateContentLabels::ID,
        pop_typed::<inspector::UpdateContentLabels>,
    ),
    (
        inspector::UpdateContentViewer::ID,
        pop_typed::<inspector::UpdateContentViewer>,
    ),
    (
        inspector::UpdateSync::ID,
        pop_typed::<inspector::UpdateSync>,
    ),
    (pad::Open::ID, pop_typed::<pad::Open>),
    (pad::Show::ID, pop_typed::<pad::Show>),
    (pad::Hide::ID, pop_typed::<pad::Hide>),
    (pad::Close::ID, pop_typed::<pad::Close>),
    (
        pad::BeginAnimation::ID,
        pop_typed::<pad::BeginAnimation>,
    ),
    (pad::EndAnimation::ID, pop_typed::<pad::EndAnimation>),
    (pad::UpdateLayout::ID, pop_typed::<pad::UpdateLayout>),
    (menu::Open::ID, pop_typed::<menu::Open>),
    (menu::Show::ID, pop_typed::<menu::Show>),
    (menu::Push::ID, pop_typed::<menu::Push>),
    (menu::Hide::ID, pop_typed::<menu::Hide>),
    (menu::Close::ID, pop_typed::<menu::Close>),
    (
        menu::UpdateVisual::ID,
        pop_typed::<menu::UpdateVisual>,
    ),
    (
        menu::UpdateLayout::ID,
        pop_typed::<menu::UpdateLayout>,
    ),
    (sitehandler::Open::ID, pop_typed::<sitehandler::Open>),
    (sitehandler::Show::ID, pop_typed::<sitehandler::Show>),
    (sitehandler::Push::ID, pop_typed::<sitehandler::Push>),
    (sitehandler::Hide::ID, pop_typed::<sitehandler::Hide>),
    (sitehandler::Close::ID, pop_typed::<sitehandler::Close>),
    (
        sitehandler::BeginAnimationInprogress::ID,
        pop_typed::<sitehandler::BeginAnimationInprogress>,
    ),
    (
        sitehandler::EndAnimationInprogress::ID,
        pop_typed::<sitehandler::EndAnimationInprogress>,
    ),
    (
        sitehandler::UpdateLayout::ID,
        pop_typed::<sitehandler::UpdateLayout>,
    ),
    (
        sitehandler::UpdateVisual::ID,
        pop_typed::<sitehandler::UpdateVisual>,
    ),
];

fn pop_command(
    ctx: Ctx,
    engine: &Arc<EngineInner>,
    id: StatusName,
) -> Result<Command, CommandError> {
    for &(command_id, dispatch) in COMMANDS {
        if command_id == id {
            return dispatch(ctx, engine);
        }
    }
    // No typed reader → can't pop it → it would stay at the queue's head forever,
    // so surface a terminal error rather than spin on an un-poppable command.
    Err(CommandError::Unknown(id))
}

/// Run one `Pop<P>` call and resolve its OUT triple, handing back the filled raw
/// struct plus the pool its inner pointers borrow from. The per-command
/// `from_raw` then converts `(P, &Pool)` into the safe struct.
fn pop<P>(
    engine: &Arc<EngineInner>,
    call: impl FnOnce(*mut P, *mut i32, *mut Ustring, *mut MempoolHandle) -> i32,
) -> Result<(P, Pool), EngineError> {
    let mut raw = MaybeUninit::<P>::uninit();
    let pool = invoke(engine, |s, e, p| call(raw.as_mut_ptr(), s, e, p))?;
    // SAFETY: on success (status == 100) the engine initialized `raw`.
    Ok((unsafe { raw.assume_init() }, pool))
}
