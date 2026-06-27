//! `frogans-player` — a desktop front-end over [`frogans_surfaces`].
//!
//! It supplies the [`Embedder`]: the winit window factory the engine calls when
//! it opens a component instance, plus the host-service callbacks (way-out URLs,
//! stop). The engine turn pump, command routing, and (later) the paint path all
//! live in `frogans-surfaces`; this binary just wires an event loop to it.

use std::path::PathBuf;

use clap::Parser;
use frogans_surfaces::{
    Embedder, Frogans, FrogansApp, ManifestConfig, Nature, SessionConfig, SurfaceKey, WindowRequest,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

/// Command-line options for the desktop player.
#[derive(Parser)]
#[command(
    name = "frogans-player",
    about = "Desktop Frogans Player over frogans-surfaces"
)]
struct Args {
    /// Path to the engine module (`fprt.dll` / `libfprt.dylib` / `libfprt.so`).
    #[arg(long)]
    engine: PathBuf,

    /// Data root directory (the four engine directories are derived under it).
    #[arg(long)]
    data: PathBuf,

    /// Frogans address to leap to on start (e.g. `frogans*demo`).
    #[arg(long)]
    address: Option<String>,

    /// Process locale.
    #[arg(long, default_value = "en_US.UTF-8")]
    locale: String,

    /// Enable the developer-tools UI.
    #[arg(long)]
    devtools: bool,
}

/// The desktop embedder: builds winit windows on request and handles host
/// services. Window painting/event-routing live in `frogans-surfaces`.
struct DesktopEmbedder {
    /// Address to leap to once the session starts (`None` ⇒ sit idle).
    home: Option<String>,
}

impl Embedder for DesktopEmbedder {
    fn started(&mut self, frogans: &mut Frogans) {
        if let Some(home) = &self.home {
            frogans.leap(home);
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop, request: &WindowRequest) -> Window {
        // The Frogans-look windows (pad / site / menu) are borderless and dragged
        // by their body (frogans-surfaces handles the gesture). The dialogs stay
        // native-decorated, so the WM gives them a titlebar to drag and a close
        // button — matching the official player.
        let borderless = matches!(
            request.key,
            SurfaceKey::Site(_) | SurfaceKey::Menu | SurfaceKey::Pad
        );
        #[allow(unused_mut)]
        let mut attributes = Window::default_attributes()
            .with_title(window_title(request.key))
            .with_decorations(!borderless)
            // Borderless windows are also transparent so the engine image's alpha
            // shows (the pad floats instead of sitting on a grey square).
            .with_transparent(borderless);

        // On Windows, a transparent swapchain composites against the window's
        // (white) redirection bitmap unless we drop it — then DWM composites
        // against the desktop instead. Without this the "transparent" areas are white.
        #[cfg(windows)]
        if borderless {
            use winit::platform::windows::WindowAttributesExtWindows;
            attributes = attributes.with_no_redirection_bitmap(true);
        }

        event_loop
            .create_window(attributes)
            .expect("failed to create window")
    }

    fn launch_way_out(&mut self, url: &str) {
        // Phase 1: log it. A later pass opens it via the platform handler.
        println!("frogans-player: launch way-out {url}");
    }

    fn stop_requested(&mut self) {
        println!("frogans-player: engine requested stop");
        // exit the process
        std::process::exit(0);
    }
}

/// A human-readable title for a surface's window.
fn window_title(key: SurfaceKey) -> String {
    match key {
        SurfaceKey::Site(id) => format!("Frogans Site {}", id.0),
        SurfaceKey::Menu => "Menu".to_string(),
        SurfaceKey::Pad => "Frogans Player".to_string(),
        SurfaceKey::Recent => "Recently Visited".to_string(),
        // `SurfaceKey` is `#[non_exhaustive]`: tolerate kinds added later.
        _ => "Frogans".to_string(),
    }
}

/// The engine target id for the host platform. Must match the engine module.
fn default_target_id() -> &'static str {
    // Only the Windows id is confirmed against the bundled `fprt.dll`; the others
    // are best guesses pending a matching engine module per platform.
    if cfg!(target_os = "windows") {
        "windows-x86-64"
    } else if cfg!(target_os = "macos") {
        "macos-x86-64"
    } else {
        "linux-x86-64"
    }
}

fn main() {
    let args = Args::parse();

    let config = SessionConfig {
        engine_module: args.engine,
        data_dir: args.data,
        locale: args.locale,
        manifest: ManifestConfig {
            target_id: default_target_id().to_string(),
            channel_id: "frogans-rs".to_string(),
            originator_id: "roobscoob".to_string(),
            comment: "frogans-player-desktop".to_string(),
            version: (0, 6, 9),
        },
        nature: Nature::Experimental,
        devtools: args.devtools,
    };

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = FrogansApp::new(config, DesktopEmbedder { home: args.address });
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("frogans-player: event loop error: {e}");
    }
}
