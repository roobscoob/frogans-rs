//! `fprt-impl` — the engine implementation, as two types.
//!
//! This is the only crate an engine author edits. An engine is an [`Engine`]
//! (process-wide state, built once at `library_initialize`) that is a factory of
//! [`Conductor`]s (one per `ctx`, built at `conductor_start` from the validated
//! [`StartInfo`] config). The [`fprt-server`] harness wraps them
//! (`Server::new::<fprt_impl::FrogansEngine>()`) and absorbs the whole C-ABI
//! lifecycle — library init, the conductor state machine, the windowed turn,
//! sleep/wake, and teardown via `Drop`. So everything here is safe, plain Rust:
//! validate the config, match the inbound [`Event`], push
//! [`Command`](fprt_server::Command)s onto [`Emit`], and return when you next want a
//! turn ([`NextWake`]).
//!
//! This is a **shell**: it accepts any config, boots, acknowledges shutdown, and
//! otherwise idles. Fill in [`FrogansEngine::new_conductor`] (validate the config,
//! build per-conductor state) and [`FrogansConductor::event`] (the real UI) — the
//! command vocabulary lives in `fprt_core::{command, component}`.
//!
//! [`fprt-server`]: https://docs.rs/fprt-server

use std::time::Duration;

use fprt_core::command::ApplicationStop;
use fprt_server::{
    Conductor, Ctx, Emit, Engine, EngineError, Event, InitError, LibraryVersion, NextWake,
    StartInfo,
};

pub mod runtime;

pub struct FrogansEngine;

impl Engine for FrogansEngine {
    fn initialize(_version: LibraryVersion) -> Result<Self, InitError> {
        Ok(FrogansEngine)
    }

    fn new_conductor(&mut self, info: StartInfo<'_>) -> Result<Box<dyn Conductor>, EngineError> {
        println!("Conductor Start: {:?}", info);

        Ok(Box::new(FrogansConductor {}))
    }
}

/// One conductor session — the engine's per-`ctx` state and behaviour. Dropped at
/// `conductor_stop`. This shell keeps only its `ctx`; a real one holds the open
/// dialogs, the current site, etc.
pub struct FrogansConductor {}

impl Conductor for FrogansConductor {
    fn event(&mut self, elapsed: Duration, event: Event<'_>, emit: &mut Emit<'_>) -> NextWake {
        match event {
            _ => unimplemented!(),
        }
    }
}
