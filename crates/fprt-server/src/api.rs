//! The engine author's surface: implement [`Engine`] (process-wide state, a factory
//! of conductors) and [`Conductor`] (one per-`ctx` session). The harness owns
//! everything else — the conductor state machine, the `ctx`→conductor map, mempool
//! accounting, and the whole C ABI.
//!
//! Teardown is `Drop`: dropping a [`Conductor`] is `conductor_stop`, dropping the
//! [`Engine`] is `library_finalize`. The harness still reports the protocol status
//! codes (bad context, wrong state) — `Drop` is only your cleanup.

use std::time::Duration;

use fprt_core::{EngineError, Event, NextWake, StartInfo};
use fprt_sys::library::initialize::INIT_FAILED;
use fprt_sys::library_version::LibraryVersion;

use crate::Emit;

/// Why [`Engine::initialize`] failed — reported to the host as `library_initialize`'s
/// `status_lib_out`. Defaults to `INIT_FAILED`; carry a specific library code with
/// [`with_status`](InitError::with_status).
#[derive(Debug, Clone, Copy)]
pub struct InitError {
    status: u32,
}

impl InitError {
    /// A generic init failure (`INIT_FAILED`).
    pub fn new() -> Self {
        InitError {
            status: INIT_FAILED,
        }
    }

    /// An init failure reporting a specific library status code.
    pub fn with_status(status: u32) -> Self {
        InitError { status }
    }

    /// The `status_lib_out` code to report.
    pub(crate) fn status(self) -> u32 {
        self.status
    }
}

impl Default for InitError {
    fn default() -> Self {
        Self::new()
    }
}

/// The engine: process-wide state and a factory of per-`ctx` [`Conductor`]s.
/// Constructed at `library_initialize`, dropped at `library_finalize`.
///
/// **Not `Send`/`Sync`** — an engine may be thread-affine (e.g. a proxy wrapping the
/// `fprt` client's single-thread `Library`). The harness drives it from the host's
/// one thread; cross-thread command staging goes through [`Emit::sender`], not the
/// engine.
pub trait Engine: 'static {
    /// `library_initialize`: build the engine. The harness has already gated the
    /// version (`BAD_VERSION`) and rejected a double-init, so this is your
    /// process-wide setup. `Err` fails the host's init.
    fn initialize(version: LibraryVersion) -> Result<Self, InitError>
    where
        Self: Sized;

    /// `conductor_start`: validate `info` and build the conductor. The harness mints
    /// and owns the host's `ctx` (the conductor *is* it), so you never see it.
    /// `Err(EngineError)` rejects the start with your code (the `conductor_start`
    /// `0x0bfb08xx` space — see `fprt_sys::conductor::start`).
    fn new_conductor(&mut self, info: StartInfo<'_>) -> Result<Box<dyn Conductor>, EngineError>;
}

/// One conductor session: the engine's per-`ctx` state and behaviour. Dropped at
/// `conductor_stop` (and at `library_finalize`). Like [`Engine`], not `Send`.
pub trait Conductor {
    /// Handle one host→engine event during a turn: emit commands on `emit`, return
    /// when you next want a turn. `elapsed` is the time the host reported at
    /// `sync_enter`.
    fn event(&mut self, elapsed: Duration, event: Event<'_>, emit: &mut Emit<'_>) -> NextWake;

    /// The host backgrounded us (`conductor_sleep_enter`).
    fn sleep(&mut self) {}

    /// The host foregrounded us (`conductor_sleep_leave`).
    fn wake(&mut self) {}
}
