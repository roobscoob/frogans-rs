//! `frogans-surfaces` — a clean, cross-platform "surface" API over the messy
//! expanse of the `fprt` conductor.
//!
//! The conductor is a single-thread *turn machine*: you `report` one event, drain
//! the engine's [`Command`] stream, and `finish` to learn when it next wants a
//! turn. It has **no concept of raw input** — it never sees a mouse coordinate;
//! the host hit-tests locally and reports only which interactive zone was
//! activated. And it drives *window* lifecycle itself, emitting per-component
//! `Open`/`Close` commands.
//!
//! This crate wraps all of that into:
//!
//!   * [`FrogansApp`] — a [`winit::application::ApplicationHandler`] the embedder
//!     runs directly (`event_loop.run_app(&mut app)`). It owns the engine, the
//!     turn pump, and (in later phases) the egui/wgpu paint path.
//!   * [`Embedder`] — the seam the embedder implements. The engine asks *it* to
//!     create the actual [`winit::window::Window`]s (so the embedder keeps full
//!     control of window attributes), and the non-window engine commands
//!     (way-out URLs, clipboard, stop…) arrive as trait callbacks.
//!
//! # Status
//!
//! Phase 1: boots the engine under the winit loop, pumps the bootstrap + idle
//! turns, and routes/logs the command stream. No surfaces are painted yet — that
//! is phase 2+ (see the crate's architecture notes).

mod app;
mod gpu;
mod session;
mod surface;

pub use app::FrogansApp;
pub use session::{Frogans, ManifestConfig, SessionConfig, SessionError};

// Re-exported so an embedder needs only a `frogans-surfaces` dependency to name
// the engine vocabulary it cares about. The whole `fprt` surface is available
// under `frogans_surfaces::fprt` for anything not re-exported here.
pub use fprt;
pub use fprt::application::Directionality;
pub use fprt::inspector::InspectorId;
pub use fprt::sitehandler::SiteId;
pub use fprt::{Command, Nature, NextWake};

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// Which conductor component instance a surface — and its backing winit window —
/// is bound to.
///
/// `Copy + Eq + Hash` so it can key the window/surface maps. `Site` carries the
/// engine [`SiteId`]; the rest are singletons for now. Non-exhaustive: more kinds
/// (inspector windows, the other chrome dialogs) arrive in later phases.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum SurfaceKey {
    /// A Frogans Site viewport (`sitehandler`). Engine-rendered visual.
    Site(SiteId),
    /// The application / context menu (`menu`). Engine-rendered visual.
    Menu,
    /// The pad window (`pad`). Engine-rendered visual.
    Pad,
    /// The recently-visited list (`recentlyvisited`). egui chrome.
    Recent,
    /// The favorites manager (`favorites`). egui chrome.
    Favorites,
    /// The blocked-addresses list (`blocked`). egui chrome.
    Blocked,
    /// The developer-directory list (`devtools`). egui chrome.
    Devtools,
    /// The recoverable-addresses list (`recovery`). egui chrome.
    Recovery,
    /// The Frogans-address entry dialog (`inputfa`). egui chrome (text field).
    Inputfa,
    /// The leap-confirmation dialog (`leaptofrogans`). egui chrome (buttons).
    Leaptofrogans,
    /// The interface-language picker (`language`). egui chrome (select list).
    Language,
    /// The zoom dialog (`zoom`). egui chrome (slider).
    Zoom,
    /// The update-available dialog (`update`). egui chrome (info + links).
    Update,
    /// The legal-information viewer (`legalinformation`). egui chrome (text).
    Legal,
    /// A per-site run inspector (`inspector`). Multi-instance, egui chrome.
    Inspector(InspectorId),
}

/// How to move a window, in the embedder's own coordinate space (screen pixels on
/// desktop, page pixels on web).
#[derive(Clone, Copy, Debug)]
pub enum WindowPosition {
    /// Move by a pixel delta — used for live dragging. The embedder applies it to
    /// the window's current position, so `frogans-surfaces` never needs to know
    /// (or track) where the window actually is.
    Relative(i32, i32),
    /// Move to an explicit position (programmatic placement, snapping).
    Absolute(i32, i32),
}

/// The engine asked the host to open a native window for a component instance.
///
/// Handed to [`Embedder::create_window`]; the embedder turns it into a concrete
/// [`Window`] with whatever attributes it likes.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct WindowRequest {
    /// Which component instance the window is for.
    pub key: SurfaceKey,
}

/// The embedder seam: window creation plus the non-window host services.
///
/// `FrogansApp` calls these while draining a turn. Window creation is required
/// (only the embedder may decide a window's attributes); every host-service
/// callback has a no-op default, so an embedder implements only what it supports.
pub trait Embedder {
    /// The session is up (engine initialized, conductor spawned, `start` pumped).
    /// Use `frogans` to drive it — e.g. [`leap`](Frogans::leap) to a home address.
    /// Default: do nothing (sit idle until some later event drives a leap).
    fn started(&mut self, _frogans: &mut Frogans) {}

    /// Create the native window for `request`. Called inline during a turn when
    /// the engine opens a component instance. The embedder owns all window
    /// attributes (including initial placement) and `event_loop` is the only thing
    /// it needs from us to build one.
    fn create_window(&mut self, event_loop: &ActiveEventLoop, request: &WindowRequest) -> Window;

    /// The engine closed a component instance; drop the matching window.
    fn window_closed(&mut self, _key: SurfaceKey) {}

    /// Size a window to its engine content (the rendered slide / pad image). The
    /// default sizes the winit window; a web embedder sizes its element instead.
    fn resize_window(&mut self, window: &Window, _key: SurfaceKey, size: (u32, u32)) {
        let _ = window.request_inner_size(PhysicalSize::new(size.0, size.1));
    }

    /// Move a window (a live drag, or programmatic placement). The default applies
    /// it to the winit window's outer position; a web embedder moves its element.
    fn reposition_window(&mut self, window: &Window, _key: SurfaceKey, to: WindowPosition) {
        match to {
            WindowPosition::Absolute(x, y) => {
                window.set_outer_position(PhysicalPosition::new(x, y));
            }
            WindowPosition::Relative(dx, dy) => {
                if let Ok(pos) = window.outer_position() {
                    window.set_outer_position(PhysicalPosition::new(pos.x + dx, pos.y + dy));
                }
            }
        }
    }

    /// Open a way-out URL externally (browser / mail client).
    fn launch_way_out(&mut self, _url: &str) {}

    /// Place text on the system clipboard.
    fn set_clipboard_text(&mut self, _text: &str) {}

    /// Place an image (straight-alpha RGBA) on the system clipboard.
    fn set_clipboard_image(&mut self, _image: &fprt::PooledImage) {}

    /// Reveal a known engine directory in the file manager (`path` is resolved by
    /// the session from the engine's directory kind).
    fn open_directory(&mut self, _path: &str) {}

    /// Re-initialize the developers directory at `path` (clear + recreate).
    fn reinitialize_developers_directory(&mut self, _path: &str) {}

    /// The engine set the UI text directionality (LTR / RTL).
    fn set_directionality(&mut self, _directionality: Directionality) {}

    /// The engine asked the host to stop. `FrogansApp` also exits the event loop.
    fn stop_requested(&mut self) {}
}
