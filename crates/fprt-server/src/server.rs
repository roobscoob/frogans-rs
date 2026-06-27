//! [`Server`] — the harness that drives an [`Engine`]/[`Conductor`] pair through the
//! full FPRT lifecycle, and the [`Emit`]/[`Sender`] sinks a conductor pushes
//! commands through.
//!
//! A `Server` wraps a [`ServerInner`] — the real state. It is **non-generic** even
//! though the engine type isn't: the engine type is captured once in a boxed
//! *construct* closure (so the global can name it), the live engine is a
//! `Box<dyn Engine>` built at `library_initialize`, and each conductor is a
//! `Box<dyn Conductor>` the harness owns alongside its [state](ConductorState) and
//! outbox. The harness owns the conductor **state machine** (rejecting out-of-state
//! calls with the real codes), so the implementor never touches it.
//!
//! `Server` is static-free — many coexist for testing — and the in-process test API
//! ([`initialize`](Server::initialize) → [`start`](Server::start) →
//! [`turn`](Server::turn) → [`sleep`](Server::sleep)/[`stop`](Server::stop)) mirrors
//! the export sequence. `into_process_engine` (in [`engine`](crate::engine)) moves
//! one inner into the process global for the FFI trampolines.

use std::collections::HashMap;
use std::time::Duration;

use fprt_core::pool::OwnedPool;
use fprt_core::{Command, Event, NextWake, StartInfo};
use fprt_sys::conductor::{sleep_enter, sleep_leave, start, stop, sync_enter, sync_leave};
use fprt_sys::ctx::Ctx;
use fprt_sys::library::initialize as lib_init;
use fprt_sys::library_version::LibraryVersion;
use fprt_core::EngineError;

use crate::api::{Conductor, Engine, InitError};
use crate::registry::Registry;
use crate::session::{Outbox, Staged};

/// Builds the live engine at `library_initialize`. Captures the engine *type* (or,
/// for [`Server::from_event_fn`], a handler closure) so [`ServerInner`] stays
/// non-generic and the process global can name it.
type Construct = Box<dyn FnMut(LibraryVersion) -> Result<Box<dyn Engine>, InitError>>;

/// Where a conductor is in its lifecycle. The harness advances it and rejects calls
/// that don't fit (the real engine's state machine, made fail-soft).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ConductorState {
    /// Live and idle — ready to enter a turn or sleep.
    Cruising,
    /// Inside a sync window (`sync_enter` … `sync_leave`): events + draining.
    InTurn,
    /// Backgrounded (`sleep_enter` … `sleep_leave`).
    Sleeping,
}

/// One live conductor: the implementor's session plus the harness bookkeeping the
/// implementor doesn't see.
pub(crate) struct ConductorEntry {
    /// The implementor's per-`ctx` session. Dropped (→ `Drop` = `conductor_stop`)
    /// when removed.
    conductor: Box<dyn Conductor>,
    /// State-machine position.
    state: ConductorState,
    /// Commands staged this window (and any pushed off-thread).
    outbox: Outbox,
    /// Time the host reported at `sync_enter`, handed to `event`.
    elapsed: Duration,
    /// Next-wake the last `event` returned, reported at `sync_leave`.
    next_wake: NextWake,
}

impl ConductorEntry {
    fn new(conductor: Box<dyn Conductor>) -> Self {
        ConductorEntry {
            conductor,
            state: ConductorState::Cruising,
            outbox: Outbox::new(),
            elapsed: Duration::ZERO,
            next_wake: NextWake::Idle,
        }
    }
}

/// The harness state — what a [`Server`] owns and what moves into the process global
/// at `into_process_engine`.
pub(crate) struct ServerInner {
    /// Builds the engine at `initialize`.
    construct: Construct,
    /// The live engine — `None` until `library_initialize`, again after `finalize`.
    engine: Option<Box<dyn Engine>>,
    /// Live conductors by `ctx`.
    conductors: HashMap<Ctx, ConductorEntry>,
    /// The mempool registry (handles + byte accounting).
    pub(crate) registry: Registry,
    /// Next `ctx` to mint.
    next_ctx: u32,
}

impl ServerInner {
    pub(crate) fn new(construct: Construct) -> Self {
        ServerInner {
            construct,
            engine: None,
            conductors: HashMap::new(),
            registry: Registry::new(),
            next_ctx: 1,
        }
    }

    // ── library lifecycle ────────────────────────────────────────────────────

    /// `library_initialize`: gate the version + double-init, then build the engine.
    /// `Err(code)` is the `status_lib_out` to report.
    pub(crate) fn initialize(&mut self, version: LibraryVersion) -> Result<(), u32> {
        if self.engine.is_some() {
            return Err(lib_init::ALREADY_INITIALIZED);
        }
        let req = LibraryVersion::REQUIRED;
        if (version.major, version.minor, version.patch) != (req.major, req.minor, req.patch) {
            return Err(lib_init::BAD_VERSION);
        }
        let engine = (self.construct)(version).map_err(InitError::status)?;
        self.engine = Some(engine);
        Ok(())
    }

    /// `library_finalize`: drop the engine (→ `Drop`). Conductors linger until the
    /// `Server` drops or they're stopped, but the library is no longer cruising.
    pub(crate) fn finalize(&mut self) {
        self.engine = None;
    }

    /// `library_is_initialized`.
    pub(crate) fn is_initialized(&self) -> bool {
        self.engine.is_some()
    }

    /// `library_report_allocated_arguments`: live handle count + total bytes.
    pub(crate) fn report_args(&self) -> (u32, u32) {
        (self.registry.live() as u32, self.registry.bytes() as u32)
    }

    // ── conductor lifecycle + state machine ──────────────────────────────────

    /// `conductor_start`: validate config + build the conductor, then mint and key
    /// it under a fresh `ctx` (which the engine never sees — the conductor *is* it).
    pub(crate) fn start(&mut self, info: StartInfo<'_>) -> Result<Ctx, EngineError> {
        let conductor = {
            let engine = self
                .engine
                .as_mut()
                .ok_or_else(|| EngineError::new(start::NOT_CRUISING, None))?;
            engine.new_conductor(info)?
        };
        let ctx = Ctx(self.next_ctx);
        self.next_ctx += 1;
        self.conductors.insert(ctx, ConductorEntry::new(conductor));
        Ok(ctx)
    }

    /// `conductor_stop`: drop the conductor (→ `Drop`).
    pub(crate) fn stop(&mut self, ctx: Ctx) -> Result<(), i32> {
        match self.conductors.remove(&ctx) {
            Some(_) => Ok(()),
            None => Err(stop::INVALID_CONTEXT),
        }
    }

    /// `conductor_sync_enter`: open a turn window, recording `elapsed`.
    pub(crate) fn sync_enter(&mut self, ctx: Ctx, elapsed: Duration) -> Result<(), i32> {
        if self.engine.is_none() {
            return Err(sync_enter::NOT_CRUISING);
        }
        let entry = self
            .conductors
            .get_mut(&ctx)
            .ok_or(sync_enter::INVALID_CONTEXT)?;
        match entry.state {
            ConductorState::InTurn => Err(sync_enter::ALREADY_ENTERED),
            ConductorState::Sleeping => Err(sync_enter::WRONG_STATE),
            ConductorState::Cruising => {
                entry.state = ConductorState::InTurn;
                entry.elapsed = elapsed;
                Ok(())
            }
        }
    }

    /// `conductor_sync_leave`: close the turn, returning the engine's next-wake.
    pub(crate) fn sync_leave(&mut self, ctx: Ctx) -> Result<NextWake, i32> {
        let entry = self
            .conductors
            .get_mut(&ctx)
            .ok_or(sync_leave::INVALID_CONTEXT)?;
        match entry.state {
            ConductorState::InTurn => {
                entry.state = ConductorState::Cruising;
                Ok(entry.next_wake)
            }
            _ => Err(sync_leave::PHASE_TOO_LOW),
        }
    }

    /// `conductor_sleep_enter`: background the conductor.
    pub(crate) fn sleep_enter(&mut self, ctx: Ctx) -> Result<(), i32> {
        if self.engine.is_none() {
            return Err(sleep_enter::NOT_CRUISING);
        }
        let entry = self
            .conductors
            .get_mut(&ctx)
            .ok_or(sleep_enter::INVALID_CONTEXT)?;
        match entry.state {
            ConductorState::Cruising => {
                entry.conductor.sleep();
                entry.state = ConductorState::Sleeping;
                Ok(())
            }
            ConductorState::Sleeping => Err(sleep_enter::ALREADY_PAUSED),
            ConductorState::InTurn => Err(sleep_enter::WRONG_STATE),
        }
    }

    /// `conductor_sleep_leave`: foreground the conductor.
    pub(crate) fn sleep_leave(&mut self, ctx: Ctx) -> Result<(), i32> {
        let entry = self
            .conductors
            .get_mut(&ctx)
            .ok_or(sleep_leave::INVALID_CONTEXT)?;
        match entry.state {
            ConductorState::Sleeping => {
                entry.conductor.wake();
                entry.state = ConductorState::Cruising;
                Ok(())
            }
            _ => Err(sleep_leave::INVALID_STATE),
        }
    }

    // ── turn internals (shared by the FFI report trampoline and `turn`) ───────

    /// Whether `ctx` is live and `engine` is up — the precondition the report
    /// trampoline checks before decoding (returns the matching `BASE+offset`).
    pub(crate) fn event_ready(&self, ctx: Ctx) -> Result<(), EventReject> {
        if self.engine.is_none() {
            return Err(EventReject::NotCruising);
        }
        match self.conductors.get(&ctx) {
            None => Err(EventReject::BadContext),
            Some(e) if e.state != ConductorState::InTurn => Err(EventReject::WrongState),
            Some(_) => Ok(()),
        }
    }

    /// Run the conductor's `event` for `ctx`, filling its outbox. Caller has already
    /// checked [`event_ready`](Self::event_ready).
    pub(crate) fn run_event(&mut self, ctx: Ctx, event: Event<'_>) {
        let entry = self.conductors.get_mut(&ctx).expect("event on a ready ctx");
        let mut emit = Emit::new(&entry.outbox);
        entry.next_wake = entry.conductor.event(entry.elapsed, event, &mut emit);
    }

    /// The next staged command's class id, for `get_next_command` — `None` if `ctx`
    /// isn't live, `Some(None)` if the queue is empty. Returns owned ids (no borrow
    /// of `self`), so the caller can still reach `registry` on the bad-`ctx` path.
    pub(crate) fn next_command_id(&self, ctx: Ctx) -> Option<Option<fprt_sys::ui::StatusName>> {
        self.conductors
            .get(&ctx)
            .map(|e| e.outbox.with_front(crate::command::status_name))
    }

    /// Take the next staged command for `ctx`'s `_pop` — `None` if `ctx` isn't live,
    /// `Some(None)` if the queue is empty.
    pub(crate) fn pop_command(&mut self, ctx: Ctx) -> Option<Option<Staged>> {
        self.conductors.get_mut(&ctx).map(|e| e.outbox.pop_front())
    }

    /// Drain every staged command for `ctx` (the in-process `turn`).
    fn drain(&mut self, ctx: Ctx) -> Vec<Command> {
        let mut commands = Vec::new();
        if let Some(entry) = self.conductors.get(&ctx) {
            while let Some(staged) = entry.outbox.pop_front() {
                commands.push(staged.command);
            }
        }
        commands
    }
}

/// Why a `_report` was rejected before the engine ran — mapped to the call's
/// `BASE + offset` by the trampoline (error-code DB §5.1).
pub(crate) enum EventReject {
    /// Library not initialized (`+3`).
    NotCruising,
    /// No such conductor (`+0`).
    BadContext,
    /// Conductor not in a turn (`+4`).
    WrongState,
}

impl EventReject {
    /// The offset off the call's block base.
    pub(crate) fn offset(self) -> u32 {
        match self {
            EventReject::BadContext => 0,
            EventReject::NotCruising => 3,
            EventReject::WrongState => 4,
        }
    }
}

/// The safe FPRT engine harness. Build one from an [`Engine`] type
/// ([`new`](Server::new)) or a handler closure ([`from_event_fn`](Server::from_event_fn)),
/// drive it in-process for tests, or install it as the process engine with
/// `into_process_engine`.
pub struct Server {
    pub(crate) inner: ServerInner,
}

impl Server {
    /// Build a harness for engine type `E`. The engine is constructed when
    /// `library_initialize` runs ([`initialize`](Server::initialize) in-process).
    pub fn new<E: Engine>() -> Self {
        Server {
            inner: ServerInner::new(Box::new(|version| {
                E::initialize(version).map(|e| Box::new(e) as Box<dyn Engine>)
            })),
        }
    }

    /// Build a harness from a stateless event handler — the quick path for tests and
    /// trivial engines (no library/conductor state; every conductor runs the same
    /// `f`). For real engines — or anything that must tell conductors apart —
    /// implement [`Engine`]/[`Conductor`] and use [`new`](Server::new).
    pub fn from_event_fn<F>(f: F) -> Self
    where
        F: FnMut(Duration, Event<'_>, &mut Emit<'_>) -> NextWake + Clone + 'static,
    {
        Server {
            inner: ServerInner::new(Box::new(move |_version| {
                Ok(Box::new(FnEngine { f: f.clone() }) as Box<dyn Engine>)
            })),
        }
    }

    /// Drive `library_initialize` with the required version (the in-process test
    /// entry; the FFI path takes the host's version). `Err` is the library code.
    pub fn initialize(&mut self) -> Result<(), u32> {
        self.inner.initialize(LibraryVersion::REQUIRED)
    }

    /// Drive `conductor_start` with `info`, returning the new conductor's `ctx`.
    pub fn start(&mut self, info: StartInfo<'_>) -> Result<Ctx, EngineError> {
        self.inner.start(info)
    }

    /// Initialize (if needed) and start one conductor with an empty config — the
    /// one-liner for tests that don't care about init/config. Panics if either step
    /// fails (a test bug for a trivial engine).
    pub fn boot(&mut self) -> Ctx {
        if !self.inner.is_initialized() {
            self.initialize().expect("library initialize");
        }
        match self.start(StartInfo::EMPTY) {
            Ok(ctx) => ctx,
            Err(e) => panic!("conductor start failed: {:#x}", e.code()),
        }
    }

    /// Drive one full window in-process: `sync_enter` → fire `event` → drain the
    /// emitted commands → `sync_leave`. The test harness — no globals, no FFI.
    pub fn turn(&mut self, ctx: Ctx, elapsed: Duration, event: Event<'_>) -> (Vec<Command>, NextWake) {
        self.inner.sync_enter(ctx, elapsed).expect("sync_enter in turn");
        self.inner.run_event(ctx, event);
        let commands = self.inner.drain(ctx);
        let wake = self.inner.sync_leave(ctx).expect("sync_leave in turn");
        (commands, wake)
    }

    /// Drive `conductor_sleep_enter`.
    pub fn sleep(&mut self, ctx: Ctx) -> Result<(), i32> {
        self.inner.sleep_enter(ctx)
    }

    /// Drive `conductor_sleep_leave`.
    pub fn wake(&mut self, ctx: Ctx) -> Result<(), i32> {
        self.inner.sleep_leave(ctx)
    }

    /// Drive `conductor_stop`.
    pub fn stop(&mut self, ctx: Ctx) -> Result<(), i32> {
        self.inner.stop(ctx)
    }
}

// ── the `from_event_fn` adapter: a stateless engine over one handler closure ──

/// A trivial [`Engine`] wrapping a handler closure; every conductor shares a clone.
/// Built only by [`Server::from_event_fn`] (its construct closure makes it
/// directly), so `initialize` is never the construction path.
struct FnEngine<F> {
    f: F,
}

impl<F> Engine for FnEngine<F>
where
    F: FnMut(Duration, Event<'_>, &mut Emit<'_>) -> NextWake + Clone + 'static,
{
    fn initialize(_version: LibraryVersion) -> Result<Self, InitError> {
        // Unreachable: `from_event_fn` builds the `FnEngine` in its construct
        // closure, never through here. `FnEngine` is private, so no other path
        // constructs it.
        unreachable!("FnEngine is built by Server::from_event_fn, not initialize")
    }

    fn new_conductor(&mut self, _info: StartInfo<'_>) -> Result<Box<dyn Conductor>, EngineError> {
        Ok(Box::new(FnConductor { f: self.f.clone() }))
    }
}

/// One conductor of a [`FnEngine`]: replays the shared handler.
struct FnConductor<F> {
    f: F,
}

impl<F> Conductor for FnConductor<F>
where
    F: FnMut(Duration, Event<'_>, &mut Emit<'_>) -> NextWake + 'static,
{
    fn event(&mut self, elapsed: Duration, event: Event<'_>, emit: &mut Emit<'_>) -> NextWake {
        (self.f)(elapsed, event, emit)
    }
}

/// The in-window command sink handed to [`Conductor::event`].
pub struct Emit<'s> {
    outbox: &'s Outbox,
}

impl<'s> Emit<'s> {
    /// Wrap a conductor's outbox as the command sink.
    pub(crate) fn new(outbox: &'s Outbox) -> Self {
        Emit { outbox }
    }
}

impl Emit<'_> {
    /// Emit a pool-free command (marker / scalar / enum): `out.command(MenuOpen)`.
    pub fn command(&mut self, command: impl Into<Command>) {
        self.outbox.push(Staged {
            command: command.into(),
            pool: crate::encoder::CallPool::new(),
        });
    }

    /// Emit a command whose data must be allocated — build it into the command's own
    /// pool: `out.command_pooled(|p| AddClipboardText::new(p, "hi").into())`.
    pub fn command_pooled(&mut self, build: impl FnOnce(&OwnedPool) -> Command) {
        let pool = crate::encoder::CallPool::new();
        let command = build(pool.arena());
        self.outbox.push(Staged { command, pool });
    }

    /// A `Send` handle to this conductor's outbox, for emitting from another thread.
    /// Off-window commands land on the next window (the host is pull-only).
    pub fn sender(&self) -> Sender {
        Sender {
            outbox: self.outbox.clone(),
        }
    }
}

/// A cross-thread command sink (a clone of the conductor's outbox).
pub struct Sender {
    outbox: Outbox,
}

impl Sender {
    /// Stage a pool-free command from any thread.
    pub fn command(&self, command: impl Into<Command>) {
        self.outbox.push(Staged {
            command: command.into(),
            pool: crate::encoder::CallPool::new(),
        });
    }

    /// Stage a command with pooled data from any thread (built into its own pool).
    pub fn command_pooled(&self, build: impl FnOnce(&OwnedPool) -> Command) {
        let pool = crate::encoder::CallPool::new();
        let command = build(pool.arena());
        self.outbox.push(Staged { command, pool });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fprt_core::command::MenuOpen;
    use fprt_core::component::application::{self, ReportStart, UpdateZoom};

    #[test]
    fn engine_emits_on_start() {
        let mut server = Server::from_event_fn(|_elapsed, event, out| {
            if let Event::ApplicationStart(start) = event {
                assert_eq!(start.locale, "en_US");
                out.command(MenuOpen);
                out.command(UpdateZoom::new(100));
            }
            NextWake::Idle
        });
        let ctx = server.boot();
        let (cmds, wake) =
            server.turn(ctx, Duration::ZERO, Event::ApplicationStart(ReportStart::new("en_US")));
        assert_eq!(wake, NextWake::Idle);
        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], Command::MenuOpen));
        assert!(matches!(cmds[1], Command::ApplicationUpdateZoom(z) if z.percent == 100));
    }

    #[test]
    fn engine_emits_pooled_command() {
        let mut server = Server::from_event_fn(|_e, event, out| {
            if matches!(event, Event::ApplicationTimeout) {
                out.command_pooled(|p| application::AddClipboardText::new(p, "réseau").into());
            }
            NextWake::In(Duration::from_millis(16))
        });
        let ctx = server.boot();
        let (cmds, wake) = server.turn(ctx, Duration::ZERO, Event::ApplicationTimeout);
        assert_eq!(wake, NextWake::In(Duration::from_millis(16)));
        match &cmds[0] {
            Command::ApplicationAddClipboardText(c) => {
                assert_eq!(c.text.as_ref().unwrap().as_str().unwrap(), "réseau");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn cross_thread_sender_lands_on_the_window() {
        let mut server = Server::from_event_fn(|_e, event, out| {
            if matches!(event, Event::ApplicationTimeout) {
                let tx = out.sender();
                std::thread::spawn(move || tx.command(MenuOpen)).join().unwrap();
            }
            NextWake::Idle
        });
        let ctx = server.boot();
        let (cmds, _) = server.turn(ctx, Duration::ZERO, Event::ApplicationTimeout);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Command::MenuOpen));
    }

    #[test]
    fn many_servers_coexist() {
        let mut a = Server::from_event_fn(|_e, _ev, out| {
            out.command(MenuOpen);
            NextWake::Idle
        });
        let mut b = Server::from_event_fn(|_e, _ev, _out| NextWake::Idle);
        let (actx, bctx) = (a.boot(), b.boot());
        let (ca, _) = a.turn(actx, Duration::ZERO, Event::ApplicationTimeout);
        let (cb, _) = b.turn(bctx, Duration::ZERO, Event::ApplicationTimeout);
        assert_eq!(ca.len(), 1);
        assert_eq!(cb.len(), 0);
    }

    #[test]
    fn state_machine_rejects_double_enter_and_recovers() {
        let mut server = Server::from_event_fn(|_e, _ev, _out| NextWake::Idle);
        let ctx = server.boot();
        // A bare enter, then a second enter is rejected — but the conductor is not
        // bricked: leave + a clean turn still work.
        server.inner.sync_enter(ctx, Duration::ZERO).unwrap();
        assert_eq!(
            server.inner.sync_enter(ctx, Duration::ZERO),
            Err(sync_enter::ALREADY_ENTERED)
        );
        server.inner.sync_leave(ctx).unwrap();
        let _ = server.turn(ctx, Duration::ZERO, Event::ApplicationTimeout);
    }

    #[test]
    fn sleep_wake_transitions() {
        let mut server = Server::from_event_fn(|_e, _ev, _out| NextWake::Idle);
        let ctx = server.boot();
        assert!(server.sleep(ctx).is_ok());
        // Can't enter a turn while sleeping; can't sleep twice.
        assert_eq!(
            server.inner.sync_enter(ctx, Duration::ZERO),
            Err(sync_enter::WRONG_STATE)
        );
        assert_eq!(server.sleep(ctx), Err(sleep_enter::ALREADY_PAUSED));
        assert!(server.wake(ctx).is_ok());
        let _ = server.turn(ctx, Duration::ZERO, Event::ApplicationTimeout);
    }
}
