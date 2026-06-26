//! Headless session driver: open an engine module, leap to an address, and pump
//! turns until the site's visual arrives.
//!
//! This mirrors the `rose-demo` example's loop, but synchronously (the engine is
//! single-threaded, so there is nothing to gain from an async runtime — we just
//! `thread::sleep` for exactly the delay the engine asks for) and with a wall
//! budget so a non-rendering address fails instead of hanging.

use std::fs;
use std::io;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use fprt::{
    Command, CommandError, ConductorConfig, Directories, EngineError, ImageFormat, Library,
    LibraryVersion, Manifest, Nature, NextWake, OpenError, Turn,
    application::{ReportLeaptofrogans, ReportStart, ReportTimeout},
    sitehandler::UpdateVisual,
};

/// Which slide of a site's visual to pull out.
#[derive(Clone, Copy)]
pub enum Slide {
    /// The expanded lead slide.
    Lead,
    /// The collapsed vignette slide.
    Vignette,
}

/// How long the engine is given to produce a site visual after the leap before
/// the render is declared failed.
const RENDER_BUDGET: Duration = Duration::from_secs(30);

/// How often to pump a timeout while the engine reports nothing pending.
const IDLE_POLL: Duration = Duration::from_millis(200);

/// Create the four engine directories under `root` (and `root` itself).
///
/// [`Directories::under`] only derives the path strings; the engine still needs
/// them to exist on disk, so we materialize the same four names here.
pub fn ensure_directories(root: &Path) -> io::Result<()> {
    for name in ["user_data", "resources", "developers", "developers_test"] {
        fs::create_dir_all(root.join(name))?;
    }
    Ok(())
}

/// Drive a fresh session to the point where the target site has been rendered,
/// returning its [`UpdateVisual`]. The returned value keeps the engine alive (its
/// pooled images pin the module from finalizing), so it stays readable after the
/// session is torn down here.
pub fn render_address(
    dll: &Path,
    data: &Path,
    address: &str,
    locale: &str,
) -> Result<UpdateVisual, SessionError> {
    let library = Library::open(dll, LibraryVersion::REQUIRED)?;

    let directories = Directories::under(data);
    let manifest = Manifest::new("windows-x86-64", "frogans-rs", "roobscoob")
        .with_comment("frogans-rs-cli")
        .with_version(0, 6, 9);

    let mut conductor = library.spawn_conductor(
        ConductorConfig::new(directories, manifest)
            .site_image_format(ImageFormat::Png)
            .standalone_image_format(ImageFormat::Png)
            .nature(Nature::Experimental),
    )?;

    // Bootstrap handshake.
    let mut turn = conductor.report(Duration::from_millis(100), ReportStart::new(locale))?;
    drain(&mut turn)?;
    turn.finish()?;

    // Leap; the visual may already be queued in the immediate response.
    let mut turn = conductor.report(Duration::from_millis(100), ReportLeaptofrogans::new(address))?;
    if let Some(visual) = take_visual(&mut turn)? {
        return Ok(visual);
    }
    let mut next = turn.finish()?;

    // Idle pump: sleep for what the engine asked, reporting real elapsed time,
    // until the site renders or the budget runs out.
    let start = Instant::now();
    let mut last = Instant::now();
    loop {
        if start.elapsed() > RENDER_BUDGET {
            return Err(SessionError::Timeout {
                address: address.to_string(),
            });
        }

        let wait = match next {
            NextWake::In(delay) => delay,
            NextWake::Idle => IDLE_POLL,
        };
        sleep(wait);

        let elapsed = last.elapsed();
        last = Instant::now();

        let mut turn = conductor.report(elapsed, ReportTimeout::new())?;
        if let Some(visual) = take_visual(&mut turn)? {
            return Ok(visual);
        }
        next = turn.finish()?;
    }
}

/// Scan a turn's commands for the site visual, returning it if present.
fn take_visual(turn: &mut Turn<'_>) -> Result<Option<UpdateVisual>, SessionError> {
    for command in turn.drain() {
        match command? {
            Command::SitehandlerUpdateVisual(visual) => return Ok(Some(visual)),
            _ => {}
        }
    }
    Ok(None)
}

/// Drain and discard a turn's commands (propagating any read error).
fn drain(turn: &mut Turn<'_>) -> Result<(), SessionError> {
    for command in turn.drain() {
        command?;
    }
    Ok(())
}

/// Anything that can go wrong driving a render session.
#[derive(Debug)]
pub enum SessionError {
    /// The engine module failed to load or initialize.
    Open(OpenError),
    /// An engine call failed.
    Engine(EngineError),
    /// Reading the command stream failed (including an unmodeled command, which
    /// stalls the queue — its id names what still needs a typed reader).
    Command(CommandError),
    /// The engine never rendered the address within the budget.
    Timeout { address: String },
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

impl From<CommandError> for SessionError {
    fn from(e: CommandError) -> Self {
        SessionError::Command(e)
    }
}

impl core::fmt::Display for SessionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SessionError::Open(e) => write!(f, "{e}"),
            SessionError::Engine(e) => write!(f, "engine call failed: {e}"),
            SessionError::Command(e) => write!(f, "reading commands failed: {e}"),
            SessionError::Timeout { address } => write!(
                f,
                "`{address}` did not render within {}s",
                RENDER_BUDGET.as_secs()
            ),
        }
    }
}

impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SessionError::Open(e) => Some(e),
            SessionError::Engine(e) => Some(e),
            SessionError::Command(e) => Some(e),
            SessionError::Timeout { .. } => None,
        }
    }
}
