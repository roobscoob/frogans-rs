//! The conductor — one live Frogans session — and its turn flow.
//!
//! [`Conductor`] is `!Send`: it owns host→engine event reporting, which must run
//! on the thread it was created on. A turn is opened with
//! [`report`](Conductor::report) — `sync_enter` + the one event, both on the main
//! thread — which hands back a `Send` [`Turn`] whose command drain and
//! `sync_leave` can run on any thread.

mod command;
mod config;
mod report;

pub mod component;

pub use command::{Command, CommandError, Commands};
pub use config::{ConductorConfig, DeploymentMode, Directories, ImageFormat, Manifest, Nature};
pub use report::Report;

use core::marker::PhantomData;
use core::time::Duration;
use std::sync::Arc;

use fprt_sys::ctx::Ctx;

use crate::call::invoke;
use crate::engine::EngineInner;
use crate::error::EngineError;

// `NextWake` lives in `fprt-core` (the server produces it, this client consumes
// it). Re-exported so `conductor::NextWake` and the public API keep resolving.
pub use fprt_core::NextWake;

/// A live conductor — one running Frogans session, from
/// [`Library::spawn_conductor`](crate::Library::spawn_conductor).
///
/// `!Send + !Sync`: the home of the engine's main-thread event API. Open a turn
/// with [`report`](Conductor::report); it returns a `Send` [`Turn`].
pub struct Conductor {
    ctx: Ctx,
    engine: Arc<EngineInner>,
    _main_thread: PhantomData<*const ()>,
}

impl Conductor {
    pub(crate) fn new(ctx: Ctx, engine: Arc<EngineInner>) -> Self {
        Conductor {
            ctx,
            engine,
            _main_thread: PhantomData,
        }
    }

    pub(crate) fn ctx(&self) -> Ctx {
        self.ctx
    }

    pub(crate) fn engine(&self) -> &Arc<EngineInner> {
        &self.engine
    }

    /// Open a turn: `sync_enter`, then fire the one `event` — both on this (the
    /// main) thread — and hand back a [`Turn`] whose command drain and
    /// `sync_leave` can run anywhere.
    ///
    /// A turn carries exactly one event; the returned `Turn` borrows the
    /// conductor, so you can't open another until it's done.
    pub fn report(
        &mut self,
        elapsed: Duration,
        event: impl Report,
    ) -> Result<Turn<'_>, EngineError> {
        sync_enter(self.ctx, &self.engine, elapsed)?;
        if let Err(e) = event.send(self) {
            // The window opened but the event failed — leave before returning so
            // the window never leaks.
            let _ = sync_leave(self.ctx, &self.engine);
            return Err(e);
        }
        Ok(Turn::new(self.ctx, Arc::clone(&self.engine)))
    }

    // temporarily disabled until we understand it better.

    // /// Pause the engine (running → paused).
    // pub fn sleep(&mut self) -> Result<(), EngineError> {
    //     let (ctx, engine) = (self.ctx, &self.engine);
    //     invoke(engine, |s, e, p| unsafe {
    //         (engine.methods().conductor_sleep_enter)(ctx, s, e, p)
    //     })
    //     .map(|_| ())
    // }

    // /// Resume the engine (paused → running).
    // pub fn wake(&mut self) -> Result<(), EngineError> {
    //     let (ctx, engine) = (self.ctx, &self.engine);
    //     invoke(engine, |s, e, p| unsafe {
    //         (engine.methods().conductor_sleep_leave)(ctx, s, e, p)
    //     })
    //     .map(|_| ())
    // }
}

impl Drop for Conductor {
    fn drop(&mut self) {
        // Best-effort teardown; the `invoke`/`check` frees stop's mempool, then
        // the engine `Arc` drops (and finalizes if it was the last).
        let (ctx, engine) = (self.ctx, &self.engine);
        // SAFETY: valid ctx; table valid via the still-live engine.
        let _ = invoke(engine, |s, e, p| unsafe {
            (engine.methods().conductor_stop)(ctx, s, e, p)
        });
    }
}

/// The command-drain + leave half of a turn, handed back by
/// [`Conductor::report`].
///
/// `Send` — move it to a worker to drain off the main thread. It borrows the
/// conductor (locking it for the window's duration) via a `PhantomData<&'a mut
/// ()>` proxy, which is what keeps it `Send` despite the conductor being `!Send`.
/// The window closes on [`finish`](Turn::finish), or on `Drop` if you don't call
/// it.
pub struct Turn<'a> {
    ctx: Ctx,
    engine: Arc<EngineInner>,
    left: bool,
    _borrow: PhantomData<&'a mut ()>,
}

impl Turn<'_> {
    fn new(ctx: Ctx, engine: Arc<EngineInner>) -> Self {
        Turn {
            ctx,
            engine,
            left: false,
            _borrow: PhantomData,
        }
    }

    /// The engine's commands for this turn — its response to the reported event.
    pub fn drain(&mut self) -> Commands<'_> {
        Commands::new(self.ctx, &self.engine)
    }

    /// End the turn (`sync_leave`), returning the next-wake delay.
    pub fn finish(mut self) -> Result<NextWake, EngineError> {
        self.left = true; // suppress the Drop leave
        sync_leave(self.ctx, &self.engine)
    }
}

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        if !self.left {
            // Best-effort leave so the window never leaks. (Poisoning the
            // conductor on a failed leave is deliberately left out for now.)
            let _ = sync_leave(self.ctx, &self.engine);
        }
    }
}

// --- the sync window endpoints ----------------------------------------------

fn sync_enter(ctx: Ctx, engine: &Arc<EngineInner>, elapsed: Duration) -> Result<(), EngineError> {
    let elapsed_ms = elapsed.as_millis().min(i32::MAX as u128) as i32;
    // SAFETY: valid ctx; table valid via the live engine.
    invoke(engine, |s, e, p| unsafe {
        (engine.methods().conductor_sync_enter)(ctx, elapsed_ms, s, e, p)
    })
    .map(|_| ())
}

fn sync_leave(ctx: Ctx, engine: &Arc<EngineInner>) -> Result<NextWake, EngineError> {
    let mut nextwake = 0u32;
    // SAFETY: valid ctx + out pointer; table valid via the live engine.
    invoke(engine, |s, e, p| unsafe {
        (engine.methods().conductor_sync_leave)(ctx, &mut nextwake, s, e, p)
    })?;
    Ok(NextWake::from_raw(nextwake))
}
