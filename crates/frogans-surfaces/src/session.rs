//! The engine session: owns the `fprt` [`Conductor`], pumps turns, and routes the
//! resulting command stream to surfaces (later phases) and to the [`Embedder`]'s
//! host-service callbacks.
//!
//! Everything here is `!Send` and lives on the winit main thread — the conductor's
//! event API must stay on its creation thread, so there is no async and no worker.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fprt::application::{
    MenuTarget, ReportLeaptofrogans, ReportMenuAccessUnwanted, ReportMenuAccessWanted, ReportQuit,
    ReportStart, ReportTimeout,
};
use fprt::menu::ReportButtonTriggered as ReportMenuButton;
use fprt::sitehandler::{ReportButtonTriggered as ReportSiteButton, ReportForceClose};
use fprt::{
    Command, Conductor, ConductorConfig, Directories, EngineError, ImageFormat, Library,
    LibraryVersion, Manifest, Nature, NextWake, OpenError, PooledImage, PooledString, Report,
};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::gpu::Gpu;
use crate::surface::{
    ChromeAction, ChromeConfig, ConfirmAction, ConfirmConfig, InputAction, InputConfig,
    InspectorAction, InspectorConfig, LanguageAction, LanguageConfig, LegalAction, LegalConfig,
    Surface, SurfaceInput, UpdateAction, UpdateConfig, ZoomAction, ZoomConfig,
};
use crate::{Embedder, SurfaceKey, WindowPosition, WindowRequest};

/// The Frogans application's manifest identity (owned form of [`Manifest`]).
///
/// [`Manifest`] borrows its strings for the spawn call only; we hold owned copies
/// in the long-lived [`SessionConfig`] and lend them at spawn time.
#[derive(Clone, Debug)]
pub struct ManifestConfig {
    /// Engine target id (e.g. `"windows-x86-64"`). Must match the engine module.
    pub target_id: String,
    /// Channel id.
    pub channel_id: String,
    /// Originator id.
    pub originator_id: String,
    /// Free-form comment.
    pub comment: String,
    /// `major.minor.patch`.
    pub version: (u32, u32, u32),
}

/// Everything needed to bring a session up. Owns its strings/paths because the
/// engine config borrows them only transiently at spawn time.
#[derive(Clone, Debug)]
pub struct SessionConfig {
    /// Path to the engine module (`fprt.dll` / `libfprt.dylib` / `libfprt.so`).
    pub engine_module: PathBuf,
    /// The data root; the four engine directories are derived under it.
    pub data_dir: PathBuf,
    /// Process locale passed to `start` (e.g. `"en_US.UTF-8"`).
    pub locale: String,
    /// The application's manifest identity.
    pub manifest: ManifestConfig,
    /// The application's nature.
    pub nature: Nature,
    /// Whether the developer-tools UI is available.
    pub devtools: bool,
}

/// A live engine session: the conductor plus the turn-timing state that lets the
/// winit loop wake exactly when the engine asks.
pub struct Session {
    conductor: Conductor,
    /// Shared wgpu context for every surface.
    gpu: Gpu,
    /// Live surfaces, keyed by the component instance they're bound to.
    surfaces: HashMap<SurfaceKey, Surface>,
    /// Reverse index: a window's id back to the surface it drives.
    by_window: HashMap<WindowId, SurfaceKey>,
    /// Addresses the embedder asked to leap to (via [`Frogans::leap`]), pumped
    /// after the embedder callback returns to avoid command-routing re-entrancy.
    pending_leaps: Vec<String>,
    /// The pad icon from `ApplicationUpdateImages`, kept so it can be applied when
    /// the pad window opens (the image command may arrive before or after `PadOpen`).
    pad_image: Option<PooledImage>,
    /// The data root, for resolving `open_directory` kinds to paths.
    data_dir: PathBuf,
    /// Pad / site loading-animation frames + inter-frame delay (from `update_images`).
    pad_anim: (Vec<PooledImage>, u64),
    site_anim: (Vec<PooledImage>, u64),
    /// Latest zoom percent (from `ApplicationUpdateZoom`), to seed the zoom dialog.
    current_zoom: Option<i32>,
    /// When the most recent turn was reported (for computing elapsed time).
    last_turn: Instant,
    /// The engine's latest next-wake request.
    next_wake: NextWake,
    /// Absolute deadline derived from [`Session::next_wake`] (`None` ⇒ idle).
    next_deadline: Option<Instant>,
}

impl Session {
    /// Bring a session up: create the engine directories, load + initialize the
    /// engine, spawn the conductor, and pump the bootstrap (`start`) turn.
    pub fn start<E: Embedder>(
        config: &SessionConfig,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) -> Result<Session, SessionError> {
        ensure_directories(&config.data_dir)?;

        // The conductor keeps the engine alive (it clones the engine `Arc`), so we
        // don't need to retain the `Library` past the spawn.
        let library = Library::open(&config.engine_module, LibraryVersion::REQUIRED)?;

        let directories = Directories::under(&config.data_dir);
        let m = &config.manifest;
        let manifest = Manifest::new(&m.target_id, &m.channel_id, &m.originator_id)
            .with_comment(&m.comment)
            .with_version(m.version.0, m.version.1, m.version.2);

        let conductor = library.spawn_conductor(
            ConductorConfig::new(directories, manifest)
                .nature(config.nature)
                .devtools(config.devtools)
                // Straight-alpha RGBA for both, so every engine image uploads to an
                // egui texture directly with no PNG decode (site slides + the pad/
                // ring/tooltip standalone images alike).
                .site_image_format(ImageFormat::Rgba)
                .standalone_image_format(ImageFormat::Rgba),
        )?;

        let mut session = Session {
            conductor,
            gpu: Gpu::new(),
            surfaces: HashMap::new(),
            by_window: HashMap::new(),
            pending_leaps: Vec::new(),
            pad_image: None,
            data_dir: config.data_dir.clone(),
            pad_anim: (Vec::new(), 0),
            site_anim: (Vec::new(), 0),
            current_zoom: None,
            last_turn: Instant::now(),
            next_wake: NextWake::Idle,
            next_deadline: None,
        };

        // Bootstrap handshake — `start` carries the process locale.
        session.report(
            Duration::from_millis(100),
            ReportStart::new(&config.locale),
            embedder,
            event_loop,
        );
        Ok(session)
    }

    /// Leap to a Frogans address (host → engine).
    pub fn leap<E: Embedder>(
        &mut self,
        address: &str,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        self.report(
            Duration::from_millis(100),
            ReportLeaptofrogans::new(address),
            embedder,
            event_loop,
        );
    }

    /// Pump any leaps the embedder queued through [`Frogans::leap`]. Called after
    /// an embedder callback returns, so command routing isn't re-entered.
    pub fn pump_pending<E: Embedder>(&mut self, embedder: &mut E, event_loop: &ActiveEventLoop) {
        if self.pending_leaps.is_empty() {
            return;
        }
        for address in std::mem::take(&mut self.pending_leaps) {
            self.leap(&address, embedder, event_loop);
        }
    }

    /// The engine's latest next-wake request.
    pub fn next_wake(&self) -> NextWake {
        self.next_wake
    }

    /// Whether the idle timer has elapsed and a `timeout` turn is due.
    pub fn timeout_due(&self) -> bool {
        matches!(self.next_deadline, Some(deadline) if Instant::now() >= deadline)
    }

    /// Pump a `timeout` turn reporting the real elapsed time since the last turn.
    pub fn pump_timeout<E: Embedder>(&mut self, embedder: &mut E, event_loop: &ActiveEventLoop) {
        let elapsed = self.last_turn.elapsed();
        self.report(elapsed, ReportTimeout::new(), embedder, event_loop);
    }

    /// Open a turn for one event, drain its commands, and record the next-wake.
    ///
    /// Commands are collected before the turn is finished and routed afterwards:
    /// a [`Command`] owns its pooled data (via the engine `Arc`), so it stays
    /// readable past `finish`, and collecting first frees the `&mut conductor`
    /// borrow the turn holds — letting routing touch `self` without aliasing.
    ///
    /// Errors are logged and swallowed rather than propagated: a single bad turn
    /// (e.g. the transient manifest-fetch failure on the first post-leap timeout)
    /// must not tear the whole app down.
    fn report<E: Embedder>(
        &mut self,
        elapsed: Duration,
        event: impl Report,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        self.last_turn = Instant::now();

        let mut commands = Vec::new();
        {
            let mut turn = match self.conductor.report(elapsed, event) {
                Ok(turn) => turn,
                Err(e) => {
                    eprintln!("frogans-surfaces: report failed: {e}");
                    return;
                }
            };
            for command in turn.drain() {
                match command {
                    Ok(c) => commands.push(c),
                    Err(e) => {
                        // Terminal for this turn's stream; the rest (if any) waits.
                        eprintln!("frogans-surfaces: command stream error: {e}");
                        break;
                    }
                }
            }
            match turn.finish() {
                Ok(wake) => self.set_wake(wake),
                Err(e) => {
                    eprintln!("frogans-surfaces: turn finish failed: {e}");
                    self.set_wake(NextWake::Idle);
                }
            }
        }

        for command in commands {
            self.route(command, embedder, event_loop);
        }
    }

    /// Deliver a window event to the surface that owns `window_id`, reporting any
    /// resulting input (a zone activation) back to the engine.
    pub fn window_event<E: Embedder>(
        &mut self,
        window_id: WindowId,
        event: WindowEvent,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        let Some(&key) = self.by_window.get(&window_id) else {
            return;
        };
        // The user closing a window is reported to the engine, which closes the
        // component (or stops) — we don't tear the window down ourselves.
        if matches!(event, WindowEvent::CloseRequested) {
            self.close_requested(key, embedder, event_loop);
            return;
        }
        let input = {
            let Some(surface) = self.surfaces.get_mut(&key) else {
                return;
            };
            let mut input = surface.on_window_event(&event);
            match event {
                WindowEvent::Resized(size) => surface.resize(&self.gpu, size),
                // A chrome dialog's click surfaces from the egui frame, not the
                // winit event, so fold the redraw's output into the input.
                WindowEvent::RedrawRequested => input = input.or(surface.redraw(&self.gpu)),
                _ => {}
            }
            input
        };

        let elapsed = self.last_turn.elapsed();
        match input {
            // A zone click reports to the component that owns the window.
            Some(SurfaceInput::Activate(button_index)) => match key {
                SurfaceKey::Site(id) => self.report(
                    elapsed,
                    ReportSiteButton::new(id, button_index, ""),
                    embedder,
                    event_loop,
                ),
                SurfaceKey::Menu => self.report(
                    elapsed,
                    ReportMenuButton::new(button_index, ""),
                    embedder,
                    event_loop,
                ),
                _ => {}
            },
            // A pad click asks for the (global) menu.
            Some(SurfaceInput::MenuAccess) => self.report(
                elapsed,
                ReportMenuAccessWanted::new(MenuTarget::Global),
                embedder,
                event_loop,
            ),
            // A chrome dialog action reports to the component that owns the window.
            Some(SurfaceInput::Chrome(action)) => {
                self.report_chrome(key, action, elapsed, embedder, event_loop)
            }
            // The address-entry dialog (`inputfa`).
            Some(SurfaceInput::Input(action)) => match action {
                InputAction::Change(text) => self.report(
                    elapsed,
                    fprt::inputfa::ReportChange::new(&text),
                    embedder,
                    event_loop,
                ),
                InputAction::Submit(text) => self.report(
                    elapsed,
                    fprt::inputfa::ReportOk::new(&text),
                    embedder,
                    event_loop,
                ),
                InputAction::Cancel => {
                    self.report(elapsed, fprt::inputfa::ReportCancel::new(), embedder, event_loop)
                }
            },
            // The leap-confirmation dialog (`leaptofrogans`).
            Some(SurfaceInput::Confirm(action)) => {
                use fprt::leaptofrogans as leap;
                match action {
                    ConfirmAction::Confirm => {
                        self.report(elapsed, leap::ReportConfirm::new(), embedder, event_loop)
                    }
                    ConfirmAction::Cancel => {
                        self.report(elapsed, leap::ReportCancel::new(), embedder, event_loop)
                    }
                    ConfirmAction::Block => {
                        self.report(elapsed, leap::ReportBlock::new(), embedder, event_loop)
                    }
                    ConfirmAction::Purge => {
                        self.report(elapsed, leap::ReportPurge::new(), embedder, event_loop)
                    }
                    ConfirmAction::Close => {
                        self.report(elapsed, leap::ReportClose::new(), embedder, event_loop)
                    }
                }
            }
            // The language picker.
            Some(SurfaceInput::Language(action)) => match action {
                LanguageAction::Choose(id) => self.report(
                    elapsed,
                    fprt::language::ReportOk::new(&id),
                    embedder,
                    event_loop,
                ),
                LanguageAction::Cancel => self.report(
                    elapsed,
                    fprt::language::ReportCancel::new(),
                    embedder,
                    event_loop,
                ),
            },
            // The zoom dialog.
            Some(SurfaceInput::Zoom(action)) => match action {
                ZoomAction::Default => {
                    self.report(elapsed, fprt::zoom::ReportOk::Default, embedder, event_loop)
                }
                ZoomAction::Apply(percent) => self.report(
                    elapsed,
                    fprt::zoom::ReportOk::Custom(percent),
                    embedder,
                    event_loop,
                ),
                ZoomAction::Cancel => {
                    self.report(elapsed, fprt::zoom::ReportCancel::new(), embedder, event_loop)
                }
            },
            // The update dialog: links are host services; cancel is an engine event.
            Some(SurfaceInput::Update(action)) => match action {
                UpdateAction::OpenUri(uri) => embedder.launch_way_out(&uri),
                UpdateAction::Cancel => {
                    self.report(elapsed, fprt::update::ReportCancel::new(), embedder, event_loop)
                }
            },
            // The legal-information viewer.
            Some(SurfaceInput::Legal(action)) => match action {
                LegalAction::Close => self.report(
                    elapsed,
                    fprt::legalinformation::ReportClose::new(),
                    embedder,
                    event_loop,
                ),
            },
            // An inspector window (the id comes from its surface key).
            Some(SurfaceInput::Inspector(action)) => {
                if let SurfaceKey::Inspector(id) = key {
                    use fprt::inspector as insp;
                    match action {
                        InspectorAction::SelectStep(i) => self.report(
                            elapsed,
                            insp::ReportStepSelected::new(id, i),
                            embedder,
                            event_loop,
                        ),
                        InspectorAction::SelectContent(i) => self.report(
                            elapsed,
                            insp::ReportContentSelected::new(id, i),
                            embedder,
                            event_loop,
                        ),
                        InspectorAction::Autosync(on) => self.report(
                            elapsed,
                            insp::ReportChangeAutosync::new(id, on),
                            embedder,
                            event_loop,
                        ),
                        InspectorAction::Synchronize => self.report(
                            elapsed,
                            insp::ReportSynchronize::new(id),
                            embedder,
                            event_loop,
                        ),
                        InspectorAction::Rerun => {
                            self.report(elapsed, insp::ReportRerun::new(id), embedder, event_loop)
                        }
                        InspectorAction::Close => {
                            self.report(elapsed, insp::ReportClose::new(id), embedder, event_loop)
                        }
                    }
                }
            }
            // A drag: ask the embedder to move the window (no engine turn).
            Some(SurfaceInput::Reposition(position)) => {
                if let Some(surface) = self.surfaces.get(&key) {
                    embedder.reposition_window(surface.window(), key, position);
                }
            }
            None => {}
        }
    }

    /// Report a window-close request to the engine per the window's kind. The
    /// engine answers with the matching `Close` (or `Stop`) command, which is what
    /// actually removes the window — keeping the engine's model authoritative.
    fn close_requested<E: Embedder>(
        &mut self,
        key: SurfaceKey,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        let elapsed = self.last_turn.elapsed();
        match key {
            SurfaceKey::Site(id) => {
                self.report(elapsed, ReportForceClose::new(id), embedder, event_loop)
            }
            SurfaceKey::Menu => {
                self.report(elapsed, ReportMenuAccessUnwanted::new(), embedder, event_loop)
            }
            // Closing the pad quits the application.
            SurfaceKey::Pad => self.report(elapsed, ReportQuit::new(), embedder, event_loop),
            // A list-dialog close is the same as its Cancel button.
            SurfaceKey::Recent
            | SurfaceKey::Favorites
            | SurfaceKey::Blocked
            | SurfaceKey::Devtools
            | SurfaceKey::Recovery => {
                self.report_chrome(key, ChromeAction::Cancel, elapsed, embedder, event_loop)
            }
            SurfaceKey::Inputfa => {
                self.report(elapsed, fprt::inputfa::ReportCancel::new(), embedder, event_loop)
            }
            SurfaceKey::Leaptofrogans => self.report(
                elapsed,
                fprt::leaptofrogans::ReportClose::new(),
                embedder,
                event_loop,
            ),
            SurfaceKey::Language => {
                self.report(elapsed, fprt::language::ReportCancel::new(), embedder, event_loop)
            }
            SurfaceKey::Zoom => {
                self.report(elapsed, fprt::zoom::ReportCancel::new(), embedder, event_loop)
            }
            SurfaceKey::Update => {
                self.report(elapsed, fprt::update::ReportCancel::new(), embedder, event_loop)
            }
            SurfaceKey::Legal => self.report(
                elapsed,
                fprt::legalinformation::ReportClose::new(),
                embedder,
                event_loop,
            ),
            SurfaceKey::Inspector(id) => self.report(
                elapsed,
                fprt::inspector::ReportClose::new(id),
                embedder,
                event_loop,
            ),
        }
    }

    /// Dispatch one drained command: window lifecycle to surfaces, host-service
    /// commands to the embedder. Visual/label *data* commands land in phase 3 —
    /// for now the rest is ignored (the stream was already proven in phase 1).
    fn route<E: Embedder>(
        &mut self,
        command: Command,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        match command {
            // --- host services -------------------------------------------------
            Command::ApplicationStop => {
                embedder.stop_requested();
                event_loop.exit();
            }
            Command::ApplicationLaunchWayOut(way_out) => {
                if let Some(url) = way_out.uri.as_ref().and_then(|s| s.as_str().ok()) {
                    embedder.launch_way_out(url);
                }
            }
            Command::ApplicationUpdateImages(images) => {
                self.pad_image = images.pad_main;
                self.pad_anim = (
                    images.pad_animation.frames.into_iter().flatten().collect(),
                    images.pad_animation.delay as u64,
                );
                self.site_anim = (
                    images.site_animation.frames.into_iter().flatten().collect(),
                    images.site_animation.delay as u64,
                );
                self.apply_pad_image(embedder);
            }
            Command::ApplicationAddClipboardText(c) => {
                if let Some(text) = c.text.as_ref().and_then(|s| s.as_str().ok()) {
                    embedder.set_clipboard_text(text);
                }
            }
            Command::ApplicationAddClipboardImage(c) => {
                if let Some(image) = &c.image {
                    embedder.set_clipboard_image(image);
                }
            }
            Command::ApplicationOpenDirectory(c) => {
                embedder.open_directory(&self.directory_path(c.kind));
            }
            Command::ApplicationReinitializeDevelopersDirectory => {
                let path = self.data_dir.join("developers").to_string_lossy().into_owned();
                embedder.reinitialize_developers_directory(&path);
            }
            Command::ApplicationUpdateDirectionality(c) => {
                embedder.set_directionality(c.directionality);
            }
            Command::ApplicationUpdateZoom(c) => {
                self.current_zoom = Some(c.percent);
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Zoom) {
                    s.set_zoom_percent(c.percent);
                }
            }

            // --- raise (Push) — bring a component's window to the front ---------
            Command::FavoritesPush => self.raise(SurfaceKey::Favorites),
            Command::RecentlyvisitedPush => self.raise(SurfaceKey::Recent),
            Command::BlockedPush => self.raise(SurfaceKey::Blocked),
            Command::DevtoolsPush => self.raise(SurfaceKey::Devtools),
            Command::ZoomPush => self.raise(SurfaceKey::Zoom),
            Command::UpdatePush => self.raise(SurfaceKey::Update),
            Command::LanguagePush => self.raise(SurfaceKey::Language),
            Command::LeaptofrogansPush => self.raise(SurfaceKey::Leaptofrogans),
            Command::InputfaPush => self.raise(SurfaceKey::Inputfa),
            Command::LegalinformationPush => self.raise(SurfaceKey::Legal),
            Command::MenuPush => self.raise(SurfaceKey::Menu),
            Command::SitehandlerPush(id) => self.raise(SurfaceKey::Site(id)),
            Command::InspectorPush(id) => self.raise(SurfaceKey::Inspector(id)),

            // --- engine-driven positioning -------------------------------------
            // This engine sends `rect: None` ("host decides"), so these are no-ops
            // here; a config that specifies a rect forwards it to the embedder.
            Command::SitehandlerUpdateLayout(l) => {
                self.reposition_engine(SurfaceKey::Site(l.id), l.rect, embedder)
            }
            Command::PadUpdateLayout(l) => {
                self.reposition_engine(SurfaceKey::Pad, l.rect, embedder)
            }
            // The engine marks these layout payloads host-discarded.
            Command::MenuUpdateLayout(_) | Command::ApplicationUpdateLayout(_) => {}

            // --- loading animations --------------------------------------------
            Command::PadBeginAnimation => self.animate(SurfaceKey::Pad, true),
            Command::PadEndAnimation => self.animate(SurfaceKey::Pad, false),
            Command::SitehandlerBeginAnimationInprogress(id) => {
                self.animate(SurfaceKey::Site(id), true)
            }
            Command::SitehandlerEndAnimationInprogress(id) => {
                self.animate(SurfaceKey::Site(id), false)
            }

            // --- pad -----------------------------------------------------------
            Command::PadOpen => self.open_surface(SurfaceKey::Pad, embedder, event_loop),
            Command::PadShow => self.set_visible(SurfaceKey::Pad, true),
            Command::PadHide => self.set_visible(SurfaceKey::Pad, false),
            Command::PadClose => self.close_surface(SurfaceKey::Pad, embedder),

            // --- menu ----------------------------------------------------------
            Command::MenuOpen => self.open_surface(SurfaceKey::Menu, embedder, event_loop),
            Command::MenuShow => self.set_visible(SurfaceKey::Menu, true),
            Command::MenuHide => self.set_visible(SurfaceKey::Menu, false),
            Command::MenuClose => self.close_surface(SurfaceKey::Menu, embedder),
            Command::MenuUpdateVisual(visual) => {
                let key = SurfaceKey::Menu;
                let size = self
                    .surfaces
                    .get_mut(&key)
                    .and_then(|surface| surface.set_menu_visual(visual));
                self.resize_surface(key, size, embedder);
            }

            // --- site windows --------------------------------------------------
            Command::SitehandlerOpen(id) => {
                self.open_surface(SurfaceKey::Site(id), embedder, event_loop)
            }
            Command::SitehandlerShow(id) => self.set_visible(SurfaceKey::Site(id), true),
            Command::SitehandlerHide(id) => self.set_visible(SurfaceKey::Site(id), false),
            Command::SitehandlerClose(id) => self.close_surface(SurfaceKey::Site(id), embedder),
            Command::SitehandlerUpdateVisual(visual) => {
                let key = SurfaceKey::Site(visual.id);
                let size = self
                    .surfaces
                    .get_mut(&key)
                    .and_then(|surface| surface.set_site_visual(visual));
                self.resize_surface(key, size, embedder);
            }
            // Engine-driven positioning is deferred to the embedder's policy; this
            // engine sends `rect: None` ("host decides") anyway. A future config
            // with a real rect would forward it as `WindowPosition::Absolute`.

            // --- recently-visited (chrome) -------------------------------------
            Command::RecentlyvisitedOpen => {
                self.open_surface(SurfaceKey::Recent, embedder, event_loop)
            }
            Command::RecentlyvisitedShow => self.set_visible(SurfaceKey::Recent, true),
            Command::RecentlyvisitedHide => self.set_visible(SurfaceKey::Recent, false),
            Command::RecentlyvisitedClose => self.close_surface(SurfaceKey::Recent, embedder),
            Command::RecentlyvisitedUpdateLabels(l) => self.configure_chrome(
                SurfaceKey::Recent,
                ChromeConfig {
                    title: pooled(&l.title),
                    can_open: true,
                    can_remove: true,
                    remove_all: pooled(&l.delete_all_button),
                    cancel: pooled(&l.cancel_button),
                },
            ),
            Command::RecentlyvisitedUpdateAddresses(a) => {
                self.set_chrome_addresses(SurfaceKey::Recent, to_strings(&a.addresses))
            }

            // --- favorites (chrome) --------------------------------------------
            Command::FavoritesOpen => self.open_surface(SurfaceKey::Favorites, embedder, event_loop),
            Command::FavoritesShow => self.set_visible(SurfaceKey::Favorites, true),
            Command::FavoritesHide => self.set_visible(SurfaceKey::Favorites, false),
            Command::FavoritesClose => self.close_surface(SurfaceKey::Favorites, embedder),
            Command::FavoritesUpdateLabels(l) => self.configure_chrome(
                SurfaceKey::Favorites,
                ChromeConfig {
                    title: pooled(&l.title),
                    can_open: true,
                    can_remove: true,
                    remove_all: pooled(&l.remove_all_button),
                    cancel: pooled(&l.cancel_button),
                },
            ),
            Command::FavoritesUpdateAddresses(a) => {
                self.set_chrome_addresses(SurfaceKey::Favorites, to_strings(&a.addresses))
            }

            // --- blocked (chrome) ----------------------------------------------
            Command::BlockedOpen => self.open_surface(SurfaceKey::Blocked, embedder, event_loop),
            Command::BlockedShow => self.set_visible(SurfaceKey::Blocked, true),
            Command::BlockedHide => self.set_visible(SurfaceKey::Blocked, false),
            Command::BlockedClose => self.close_surface(SurfaceKey::Blocked, embedder),
            Command::BlockedUpdateLabels(l) => self.configure_chrome(
                SurfaceKey::Blocked,
                ChromeConfig {
                    title: pooled(&l.title),
                    can_open: false, // blocked addresses can only be removed
                    can_remove: true,
                    remove_all: pooled(&l.remove_all_button),
                    cancel: pooled(&l.close_button),
                },
            ),
            Command::BlockedUpdateAddresses(a) => {
                self.set_chrome_addresses(SurfaceKey::Blocked, to_strings(&a.addresses))
            }

            // --- devtools (chrome) ---------------------------------------------
            Command::DevtoolsOpen => self.open_surface(SurfaceKey::Devtools, embedder, event_loop),
            Command::DevtoolsShow => self.set_visible(SurfaceKey::Devtools, true),
            Command::DevtoolsHide => self.set_visible(SurfaceKey::Devtools, false),
            Command::DevtoolsClose => self.close_surface(SurfaceKey::Devtools, embedder),
            Command::DevtoolsUpdateLabels(l) => self.configure_chrome(
                SurfaceKey::Devtools,
                ChromeConfig {
                    title: pooled(&l.title),
                    can_open: true, // row click = inspect
                    can_remove: false,
                    remove_all: None,
                    cancel: pooled(&l.cancel_button),
                },
            ),
            Command::DevtoolsUpdateAddresses(a) => {
                self.set_chrome_addresses(SurfaceKey::Devtools, to_strings(&a.addresses))
            }

            // --- recovery (chrome) ---------------------------------------------
            Command::RecoveryOpen => self.open_surface(SurfaceKey::Recovery, embedder, event_loop),
            Command::RecoveryShow => self.set_visible(SurfaceKey::Recovery, true),
            Command::RecoveryHide => self.set_visible(SurfaceKey::Recovery, false),
            Command::RecoveryClose => self.close_surface(SurfaceKey::Recovery, embedder),
            Command::RecoveryUpdateLabels(l) => self.configure_chrome(
                SurfaceKey::Recovery,
                ChromeConfig {
                    title: pooled(&l.title),
                    can_open: true,
                    can_remove: false,
                    remove_all: None,
                    cancel: pooled(&l.cancel_button),
                },
            ),
            Command::RecoveryUpdateAddresses(a) => {
                self.set_chrome_addresses(SurfaceKey::Recovery, to_strings(&a.addresses))
            }

            // --- inputfa (address entry) ---------------------------------------
            Command::InputfaOpen => self.open_surface(SurfaceKey::Inputfa, embedder, event_loop),
            Command::InputfaShow => self.set_visible(SurfaceKey::Inputfa, true),
            Command::InputfaHide => self.set_visible(SurfaceKey::Inputfa, false),
            Command::InputfaClose => self.close_surface(SurfaceKey::Inputfa, embedder),
            Command::InputfaUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inputfa) {
                    s.set_input_config(InputConfig {
                        title: pooled(&l.window_title),
                        instruction: pooled(&l.instruction),
                        placeholder: pooled(&l.input_placeholder),
                        ok: pooled(&l.ok_button_title),
                        cancel: pooled(&l.close_button_title),
                    });
                }
            }
            Command::InputfaUpdateAddress(a) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inputfa) {
                    s.set_input_text(pooled(&a.address).unwrap_or_default());
                }
            }
            Command::InputfaUpdateErrorRaise(e) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inputfa) {
                    s.set_input_error(pooled(&e.error_msg));
                }
            }
            Command::InputfaUpdateErrorClear => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inputfa) {
                    s.set_input_error(None);
                }
            }

            // --- leaptofrogans (leap confirmation) -----------------------------
            Command::LeaptofrogansOpen => {
                self.open_surface(SurfaceKey::Leaptofrogans, embedder, event_loop)
            }
            Command::LeaptofrogansShow => self.set_visible(SurfaceKey::Leaptofrogans, true),
            Command::LeaptofrogansHide => self.set_visible(SurfaceKey::Leaptofrogans, false),
            Command::LeaptofrogansClose => self.close_surface(SurfaceKey::Leaptofrogans, embedder),
            Command::LeaptofrogansUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Leaptofrogans) {
                    s.set_confirm_config(ConfirmConfig {
                        title: pooled(&l.title),
                        instruction: pooled(&l.instruction),
                        confirm: pooled(&l.confirm_button),
                        cancel: pooled(&l.cancel_button),
                        block: pooled(&l.block_button),
                        purge: pooled(&l.purge_button),
                        close: pooled(&l.close_button),
                    });
                }
            }
            Command::LeaptofrogansUpdateAddress(a) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Leaptofrogans) {
                    s.set_confirm_address(pooled(&a.address), a.compliant);
                }
            }

            // --- language (select list) ----------------------------------------
            Command::LanguageOpen => self.open_surface(SurfaceKey::Language, embedder, event_loop),
            Command::LanguageShow => self.set_visible(SurfaceKey::Language, true),
            Command::LanguageHide => self.set_visible(SurfaceKey::Language, false),
            Command::LanguageClose => self.close_surface(SurfaceKey::Language, embedder),
            Command::LanguageUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Language) {
                    s.set_language_config(LanguageConfig {
                        title: pooled(&l.title),
                        cancel: pooled(&l.cancel_button),
                    });
                }
            }
            Command::LanguageUpdateList(list) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Language) {
                    let languages = list
                        .languages
                        .iter()
                        .map(|lang| {
                            (
                                pooled(&lang.identifier).unwrap_or_default(),
                                pooled(&lang.name).unwrap_or_default(),
                            )
                        })
                        .collect();
                    s.set_language_list(languages, pooled(&list.current));
                }
            }

            // --- zoom ----------------------------------------------------------
            Command::ZoomOpen => {
                self.open_surface(SurfaceKey::Zoom, embedder, event_loop);
                if let Some(percent) = self.current_zoom
                    && let Some(s) = self.surfaces.get_mut(&SurfaceKey::Zoom)
                {
                    s.set_zoom_percent(percent);
                }
            }
            Command::ZoomShow => self.set_visible(SurfaceKey::Zoom, true),
            Command::ZoomHide => self.set_visible(SurfaceKey::Zoom, false),
            Command::ZoomClose => self.close_surface(SurfaceKey::Zoom, embedder),
            Command::ZoomUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Zoom) {
                    s.set_zoom_config(ZoomConfig {
                        title: pooled(&l.title),
                        default_label: pooled(&l.default_button),
                        ok: pooled(&l.ok_button),
                        cancel: pooled(&l.cancel_button),
                    });
                }
            }

            // --- update (info + links) -----------------------------------------
            Command::UpdateOpen => self.open_surface(SurfaceKey::Update, embedder, event_loop),
            Command::UpdateShow => self.set_visible(SurfaceKey::Update, true),
            Command::UpdateHide => self.set_visible(SurfaceKey::Update, false),
            Command::UpdateClose => self.close_surface(SurfaceKey::Update, embedder),
            Command::UpdateUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Update) {
                    s.set_update_config(UpdateConfig {
                        title: pooled(&l.window_title),
                        instruction: pooled(&l.instruction_text),
                        download: pooled(&l.download_button_title),
                        cancel: pooled(&l.cancel_button_title),
                    });
                }
            }
            Command::UpdateUpdateData(d) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Update) {
                    s.set_update_uris(pooled(&d.update_uri), pooled(&d.changed_branch_uri));
                }
            }

            // --- legalinformation (text viewer) --------------------------------
            Command::LegalinformationOpen => {
                self.open_surface(SurfaceKey::Legal, embedder, event_loop)
            }
            Command::LegalinformationShow => self.set_visible(SurfaceKey::Legal, true),
            Command::LegalinformationHide => self.set_visible(SurfaceKey::Legal, false),
            Command::LegalinformationClose => self.close_surface(SurfaceKey::Legal, embedder),
            Command::LegalinformationUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Legal) {
                    s.set_legal_config(LegalConfig {
                        title: pooled(&l.title),
                        close: pooled(&l.close_button),
                    });
                }
            }
            Command::LegalinformationUpdateLegalContent(c) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Legal) {
                    // Flatten the default-language document's topics to (title, text).
                    let doc = c
                        .documents
                        .get(c.default_language as usize)
                        .or_else(|| c.documents.first());
                    let topics = doc
                        .map(|d| {
                            d.topics
                                .iter()
                                .map(|t| {
                                    (
                                        pooled(&t.title).unwrap_or_default(),
                                        strip_html(&pooled(&t.html).unwrap_or_default()),
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    s.set_legal_topics(topics);
                }
            }

            // --- inspector (multi-instance run inspector) ----------------------
            Command::InspectorOpen(id) => {
                self.open_surface(SurfaceKey::Inspector(id), embedder, event_loop)
            }
            Command::InspectorShow(id) => self.set_visible(SurfaceKey::Inspector(id), true),
            Command::InspectorHide(id) => self.set_visible(SurfaceKey::Inspector(id), false),
            Command::InspectorClose(id) => self.close_surface(SurfaceKey::Inspector(id), embedder),
            Command::InspectorUpdateLabels(l) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(l.id)) {
                    s.set_inspector_config(InspectorConfig {
                        title: pooled(&l.title),
                        synchronize: pooled(&l.synchronize_button),
                        rerun: pooled(&l.rerun_button_reload),
                        close: Some("Close".to_string()),
                    });
                }
            }
            Command::InspectorUpdateAddress(a) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(a.id)) {
                    s.set_inspector_address(pooled(&a.address));
                }
            }
            Command::InspectorUpdateStatus(st) => {
                use fprt::inspector::RunStatus;
                let text = match st.run_status {
                    RunStatus::Completed => Some("Completed".to_string()),
                    RunStatus::RejectionRaised => Some("Rejection raised".to_string()),
                    _ => None,
                };
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(st.id)) {
                    s.set_inspector_status(text);
                }
            }
            Command::InspectorUpdateStepsLabels(st) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(st.id)) {
                    s.set_inspector_steps(to_strings(&st.labels), st.active_step);
                }
            }
            Command::InspectorUpdateContentLabels(cl) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(cl.id)) {
                    s.set_inspector_content_labels(to_strings(&cl.labels), cl.content_active);
                }
            }
            Command::InspectorUpdateContentViewer(cv) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(cv.id)) {
                    s.set_inspector_content(pooled(&cv.content).unwrap_or_default());
                }
            }
            Command::InspectorUpdateSync(sy) => {
                if let Some(s) = self.surfaces.get_mut(&SurfaceKey::Inspector(sy.id)) {
                    s.set_inspector_sync(sy.autosync_on, sy.synchronize_enabled);
                }
            }

            // Other components arrive later.
            _ => {}
        }
    }

    /// Open a surface for `key`: ask the embedder to build the window, bind a
    /// surface to it, and index it both ways. A no-op if already open.
    fn open_surface<E: Embedder>(
        &mut self,
        key: SurfaceKey,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        if self.surfaces.contains_key(&key) {
            return;
        }
        let window = Arc::new(embedder.create_window(event_loop, &WindowRequest { key }));
        let window_id = window.id();
        let surface = Surface::new(key, window, &mut self.gpu);
        self.by_window.insert(window_id, key);
        self.surfaces.insert(key, surface);
        if key == SurfaceKey::Pad {
            self.apply_pad_image(embedder);
        }
    }

    /// Apply the stored pad icon to the pad surface, if both exist. Called when
    /// either the image or the pad window arrives (order isn't guaranteed).
    fn apply_pad_image<E: Embedder>(&mut self, embedder: &mut E) {
        let size = match (&self.pad_image, self.surfaces.get_mut(&SurfaceKey::Pad)) {
            (Some(image), Some(surface)) => surface.set_pad_image(image),
            _ => None,
        };
        self.resize_surface(SurfaceKey::Pad, size, embedder);
    }

    /// Ask the embedder to size a surface's window to its engine content.
    fn resize_surface<E: Embedder>(
        &self,
        key: SurfaceKey,
        size: Option<(u32, u32)>,
        embedder: &mut E,
    ) {
        if let (Some(size), Some(surface)) = (size, self.surfaces.get(&key)) {
            embedder.resize_window(surface.window(), key, size);
        }
    }

    /// Set a chrome dialog's title + action configuration (if it's open).
    fn configure_chrome(&mut self, key: SurfaceKey, config: ChromeConfig) {
        if let Some(surface) = self.surfaces.get_mut(&key) {
            surface.set_chrome_config(config);
        }
    }

    /// Set a chrome dialog's address rows (if it's open).
    fn set_chrome_addresses(&mut self, key: SurfaceKey, addresses: Vec<String>) {
        if let Some(surface) = self.surfaces.get_mut(&key) {
            surface.set_chrome_addresses(addresses);
        }
    }

    /// Map a chrome dialog action to the owning component's host→engine event.
    /// `(key, action)` combinations a component doesn't support are no-ops.
    fn report_chrome<E: Embedder>(
        &mut self,
        key: SurfaceKey,
        action: ChromeAction,
        elapsed: Duration,
        embedder: &mut E,
        event_loop: &ActiveEventLoop,
    ) {
        use fprt::{blocked, devtools, favorites, recentlyvisited as recent, recovery};
        use ChromeAction::{Cancel, Open, Remove, RemoveAll};

        match (key, action) {
            (SurfaceKey::Recent, Open(a)) => {
                self.report(elapsed, recent::ReportOpen::new(&[a.as_str()]), embedder, event_loop)
            }
            (SurfaceKey::Recent, Remove(a)) => self.report(
                elapsed,
                recent::ReportDelete::new(&[a.as_str()]),
                embedder,
                event_loop,
            ),
            (SurfaceKey::Recent, RemoveAll) => {
                self.report(elapsed, recent::ReportDeleteAll::new(), embedder, event_loop)
            }
            (SurfaceKey::Recent, Cancel) => {
                self.report(elapsed, recent::ReportCancel::new(), embedder, event_loop)
            }

            (SurfaceKey::Favorites, Open(a)) => self.report(
                elapsed,
                favorites::ReportOpen::new(&[a.as_str()]),
                embedder,
                event_loop,
            ),
            (SurfaceKey::Favorites, Remove(a)) => self.report(
                elapsed,
                favorites::ReportRemove::new(&[a.as_str()]),
                embedder,
                event_loop,
            ),
            (SurfaceKey::Favorites, RemoveAll) => {
                self.report(elapsed, favorites::ReportRemoveAll::new(), embedder, event_loop)
            }
            (SurfaceKey::Favorites, Cancel) => {
                self.report(elapsed, favorites::ReportCancel::new(), embedder, event_loop)
            }

            (SurfaceKey::Blocked, Remove(a)) => self.report(
                elapsed,
                blocked::ReportRemove::new(&[a.as_str()]),
                embedder,
                event_loop,
            ),
            (SurfaceKey::Blocked, RemoveAll) => {
                self.report(elapsed, blocked::ReportRemoveAll::new(), embedder, event_loop)
            }
            (SurfaceKey::Blocked, Cancel) => {
                self.report(elapsed, blocked::ReportCancel::new(), embedder, event_loop)
            }

            (SurfaceKey::Devtools, Open(a)) => self.report(
                elapsed,
                devtools::ReportInspect::new(&[a.as_str()]),
                embedder,
                event_loop,
            ),
            (SurfaceKey::Devtools, Cancel) => {
                self.report(elapsed, devtools::ReportCancel::new(), embedder, event_loop)
            }

            (SurfaceKey::Recovery, Open(a)) => self.report(
                elapsed,
                recovery::ReportOpen::new(&[a.as_str()]),
                embedder,
                event_loop,
            ),
            (SurfaceKey::Recovery, Cancel) => {
                self.report(elapsed, recovery::ReportCancel::new(), embedder, event_loop)
            }

            // Unsupported combinations (e.g. a click on a blocked row).
            _ => {}
        }
    }

    /// Resolve an engine `open_directory` kind to a host path under the data root.
    fn directory_path(&self, kind: fprt::application::OpenDirKind) -> String {
        use fprt::application::OpenDirKind;
        let dir = match kind {
            OpenDirKind::Developers => self.data_dir.join("developers"),
            // Default / unknown ⇒ the data root.
            _ => self.data_dir.clone(),
        };
        dir.to_string_lossy().into_owned()
    }

    /// Forward an engine-specified position to the embedder (no-op if the rect is
    /// `None` — "host decides" — or the surface isn't open).
    fn reposition_engine<E: Embedder>(
        &self,
        key: SurfaceKey,
        rect: Option<fprt::visual::ScreenRect>,
        embedder: &mut E,
    ) {
        if let (Some(r), Some(surface)) = (rect, self.surfaces.get(&key)) {
            embedder.reposition_window(surface.window(), key, WindowPosition::Absolute(r.x, r.y));
        }
    }

    /// Raise an open surface's window to the front (a no-op if it isn't open).
    fn raise(&self, key: SurfaceKey) {
        if let Some(surface) = self.surfaces.get(&key) {
            surface.raise();
        }
    }

    /// Start or stop a surface's loading animation: the pad's icon cycle, or a
    /// site's in-progress spinner. Frames come from `update_images`.
    fn animate(&mut self, key: SurfaceKey, on: bool) {
        let (frames, delay) = if key == SurfaceKey::Pad {
            &self.pad_anim
        } else {
            &self.site_anim
        };
        let delay = *delay;
        let refs: Vec<(&[u8], u32, u32)> = frames
            .iter()
            .map(|i| (i.bytes(), i.width(), i.height()))
            .collect();
        let Some(surface) = self.surfaces.get_mut(&key) else {
            return;
        };
        match (key, on) {
            (SurfaceKey::Pad, true) => surface.start_pad_animation(&refs, delay),
            (SurfaceKey::Pad, false) => surface.stop_pad_animation(),
            (_, true) => surface.start_spinner(&refs, delay),
            (_, false) => surface.stop_spinner(),
        }
    }

    /// Show or hide an open surface's window (a no-op if it isn't open).
    fn set_visible(&self, key: SurfaceKey, visible: bool) {
        if let Some(surface) = self.surfaces.get(&key) {
            surface.set_visible(visible);
        }
    }

    /// Close and drop a surface, telling the embedder its window is gone.
    fn close_surface<E: Embedder>(&mut self, key: SurfaceKey, embedder: &mut E) {
        if let Some(surface) = self.surfaces.remove(&key) {
            self.by_window.remove(&surface.window_id());
            embedder.window_closed(key);
        }
    }

    /// Queue a leap to be pumped after the current embedder callback returns.
    fn request_leap(&mut self, address: &str) {
        self.pending_leaps.push(address.to_string());
    }

    /// Record the next-wake and its absolute deadline.
    fn set_wake(&mut self, wake: NextWake) {
        self.next_wake = wake;
        self.next_deadline = match wake {
            NextWake::In(delay) => Some(Instant::now() + delay),
            NextWake::Idle => None,
        };
    }
}

/// A handle the embedder uses to drive the session from its callbacks (today:
/// leap to an address). Requests are queued and pumped once the callback returns,
/// so issuing one never re-enters command routing.
///
/// Handed to [`Embedder::started`](crate::Embedder::started); phase 4 widens it
/// with the input-driven events (zone activation, entry text, …).
pub struct Frogans<'a> {
    session: &'a mut Session,
}

impl<'a> Frogans<'a> {
    pub(crate) fn new(session: &'a mut Session) -> Self {
        Frogans { session }
    }

    /// Leap to a Frogans address (queued; pumped after this callback returns).
    pub fn leap(&mut self, address: &str) {
        self.session.request_leap(address);
    }
}

/// Convert an optional pooled (engine-owned) string into an owned `String`.
fn pooled(s: &Option<PooledString>) -> Option<String> {
    s.as_ref().and_then(|s| s.as_str().ok()).map(str::to_string)
}

/// Convert an engine address list into owned strings (dropping any non-UTF-8).
fn to_strings(addresses: &[PooledString]) -> Vec<String> {
    addresses
        .iter()
        .filter_map(|s| s.as_str().ok())
        .map(str::to_string)
        .collect()
}

/// Strip HTML tags from legal-topic content (egui has no HTML renderer, so the
/// viewer shows plain text). Drops everything between `<` and `>`.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Create the four engine directories under `root` (and `root` itself).
///
/// [`Directories::under`] only derives the path strings; the engine still needs
/// them to exist on disk.
fn ensure_directories(root: &Path) -> io::Result<()> {
    for name in ["user_data", "resources", "developers", "developers_test"] {
        fs::create_dir_all(root.join(name))?;
    }
    Ok(())
}

/// Anything that can go wrong bringing a [`Session`] up.
#[derive(Debug)]
pub enum SessionError {
    /// A required engine directory could not be created.
    Io(io::Error),
    /// The engine module failed to load or initialize.
    Open(OpenError),
    /// Spawning the conductor failed.
    Engine(EngineError),
}

impl From<io::Error> for SessionError {
    fn from(e: io::Error) -> Self {
        SessionError::Io(e)
    }
}

impl From<OpenError> for SessionError {
    fn from(e: OpenError) -> Self {
        SessionError::Open(e)
    }
}

impl From<EngineError> for SessionError {
    fn from(e: EngineError) -> Self {
        SessionError::Engine(e)
    }
}

impl core::fmt::Display for SessionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SessionError::Io(e) => write!(f, "creating engine directories failed: {e}"),
            SessionError::Open(e) => write!(f, "{e}"),
            SessionError::Engine(e) => write!(f, "spawning the conductor failed: {e}"),
        }
    }
}

impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SessionError::Io(e) => Some(e),
            SessionError::Open(e) => Some(e),
            SessionError::Engine(e) => Some(e),
        }
    }
}
