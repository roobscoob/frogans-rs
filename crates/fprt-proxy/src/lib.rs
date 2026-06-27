//! `fprt_proxy.dll` — a debugging proxy engine.
//!
//! It loads the *real* engine sitting next to it as `source-fprt.dll` and sits
//! between it and the host: **inbound** it's an [`Engine`] the EXE drives (via
//! fprt-server), **outbound** it's a [`fprt::Library`] client driving the real
//! engine. Everything crosses the safe typed layer in the middle, so the proxy can
//! log/inspect fully-decoded [`Event`]s and `Command`s — and, as a bonus, every
//! field round-trips through our codec against the *real* engine + EXE, a live
//! correctness check.
//!
//! Deploy: copy `fprt_proxy.dll` → `fprt.dll`, and the real engine → `source-fprt.dll`.
//!
//! ## Status: forwarding end to end
//!
//! Inbound events are forwarded to source in one call (`impl Report for Event` in
//! the client), and source's commands are drained, logged, and re-emitted to the
//! EXE. Re-emit is sound because [`Command::copy_into`] deep-copies each command's
//! pooled bytes out of source's foreign mempool — freed when the source turn ends —
//! into the command's own pool before it reaches the host.
//!
//! `new_conductor` forwards source's boot config faithfully — pixel formats,
//! deployment mode, nature, and the devtools / exit-button flags are decoded raw →
//! typed and applied (unknown raw values are logged, not guessed). The single
//! unforwarded selector is `reserved_flag`, which the client's `ConductorConfig`
//! always pins to `ReservedFlag::SECOND`; its inbound value is logged for visibility.

use std::time::Duration;

use fprt::{ConductorConfig, DeploymentMode, Directories, ImageFormat, Library, Manifest, Nature};
use fprt_server::{
    Conductor, Emit, Engine, EngineError, Event, InitError, LibraryVersion, NextWake, Server,
    StartInfo,
};

/// Install the proxy as the process engine at library load (before the EXE calls
/// any export), exactly like the production cdylib's ctor.
#[ctor::ctor]
fn install_proxy() {
    attach_debug_console();
    let server = Server::new::<ProxyEngine>();
    if let Ok(table) = server.into_process_engine() {
        let _ = fprt_exports::install(table);
    }
}

/// The proxy engine: holds the real engine, loaded as a client. `!Send` (it holds
/// the thread-affine [`fprt::Library`]) — which the `!Send` [`Engine`] trait now
/// allows.
pub struct ProxyEngine {
    source: Library,
}

impl Engine for ProxyEngine {
    fn initialize(version: LibraryVersion) -> Result<Self, InitError> {
        // The real engine is `binaries/source-fprt.dll`, relative to the host's
        // working directory (the EXE's own folder — `player/` in our layout).
        let path = "binaries/source-fprt.dll";
        match Library::open(path, version) {
            Ok(source) => {
                eprintln!("[fprt-proxy] loaded {path}");
                Ok(ProxyEngine { source })
            }
            Err(e) => {
                eprintln!("[fprt-proxy] FAILED to load {path}: {e}");
                Err(InitError::new())
            }
        }
    }

    fn new_conductor(&mut self, info: StartInfo<'_>) -> Result<Box<dyn Conductor>, EngineError> {
        eprintln!(
            "[fprt-proxy] conductor_start: target={:?} channel={:?} origin={:?}",
            info.manifest_target_id, info.manifest_channel_id, info.manifest_originator_id
        );
        let directories = Directories::new(
            info.user_data,
            info.fonts, // the resources dir (holds the engine fonts)
            info.developers,
            info.developers_test,
        );
        let manifest = Manifest::new(
            info.manifest_target_id,
            info.manifest_channel_id,
            info.manifest_originator_id,
        )
        .with_comment(info.manifest_comment)
        .with_version(
            info.manifest_ver_major,
            info.manifest_ver_minor,
            info.manifest_ver_patch,
        );
        // Forward the host's pixel-format / deployment-mode / nature / support
        // selectors faithfully, rather than letting `ConductorConfig` substitute its
        // defaults. Each multi-variant enum is decoded raw → typed; an unrecognized
        // raw value is logged and left at the default (never guessed). The two
        // support flags decode to the `bool`s the config takes.
        let mut config = ConductorConfig::new(directories, manifest)
            .devtools(info.devtools_enabled())
            .exit_button(info.exit_button_present());
        match ImageFormat::try_from(info.imgfmt_a) {
            Ok(f) => config = config.standalone_image_format(f),
            Err(raw) => eprintln!("[fprt-proxy] unknown standalone image format {raw:?}; keeping default"),
        }
        match ImageFormat::try_from(info.imgfmt_b) {
            Ok(f) => config = config.site_image_format(f),
            Err(raw) => eprintln!("[fprt-proxy] unknown site image format {raw:?}; keeping default"),
        }
        match Nature::try_from(info.nature) {
            Ok(n) => config = config.nature(n),
            Err(raw) => eprintln!("[fprt-proxy] unknown nature {raw:?}; keeping default"),
        }
        match DeploymentMode::try_from(info.deployment_mode) {
            Ok(m) => config = config.deployment_mode(m),
            Err(raw) => eprintln!("[fprt-proxy] unknown deployment mode {raw:?}; keeping default"),
        }
        // `reserved_flag` is the one selector `ConductorConfig` can't carry — the
        // client always sends `ReservedFlag::SECOND` (the expected value), so it's
        // not forwarded. Surface the inbound value so a mismatch is at least visible.
        eprintln!("[fprt-proxy] reserved_flag (not forwarded): {:?}", info.reserved_flag);

        println!("[fprt-proxy] spawn_conductor: {config:?}");

        let source = self.source.spawn_conductor(config)?;
        Ok(Box::new(ProxyConductor { source }))
    }
}

/// One proxied conductor: wraps the real engine's conductor. Dropping it runs
/// source's `conductor_stop` (the client's `Drop`), so teardown chains for free.
pub struct ProxyConductor {
    source: fprt::Conductor,
}

impl Conductor for ProxyConductor {
    fn event(&mut self, elapsed: Duration, event: Event<'_>, emit: &mut Emit<'_>) -> NextWake {
        eprintln!("[fprt-proxy] → {event:?}");

        // Forward the event to source (eager per-event: one source turn) — enabler 1
        // (`impl Report for Event` in the client) makes this a single call.
        match self.source.report(elapsed, event) {
            Ok(mut turn) => {
                // Drain source's commands, log each, and re-emit it to the EXE. The
                // drained command's pooled bytes still live in *source's* foreign
                // mempool, which is freed when this turn ends — so we can't forward it
                // by move. `Command::copy_into` deep-copies those bytes into the
                // command's own pool (allocated by `command_pooled`), so the re-emit
                // outlives source's pool. Pool-free commands copy through untouched.
                for cmd in turn.drain() {
                    match cmd {
                        Ok(c) => {
                            eprintln!("[fprt-proxy] ← {c:?}");
                            emit.command_pooled(|p| c.copy_into(p));
                        }
                        Err(e) => eprintln!("[fprt-proxy] ← command error: {e}"),
                    }
                }
                turn.finish().unwrap_or(NextWake::Idle)
            }
            Err(e) => {
                eprintln!("[fprt-proxy] forward failed: {e}");
                NextWake::Idle
            }
        }
    }
}

/// On Windows, attach a console at load so the proxy's traces are visible (a DLL
/// loaded by a GUI host inherits no console). Unlike the production `fprt.dll`
/// (which gates this to `debug_assertions`), the proxy attaches it **always** — it's
/// a debugging artifact by nature, never shipped, so a `--release` proxy build still
/// shows traffic. No-op on other platforms (they keep their terminal).
#[cfg(windows)]
fn attach_debug_console() {
    // SAFETY: `AllocConsole` takes no args and is always safe to call (it returns 0
    // if a console already exists, e.g. when launched from a terminal).
    unsafe extern "system" {
        fn AllocConsole() -> i32;
    }
    unsafe {
        AllocConsole();
    }
    eprintln!("[fprt-proxy] console attached");
}

#[cfg(not(windows))]
fn attach_debug_console() {}
