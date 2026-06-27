//! `fprt-server` — safe scaffolding for *implementing* an FPRT engine.
//!
//! The mirror image of `fprt` (the client): where the client wraps an [`Fprt`]
//! table into a safe caller API, this turns safe Rust *behaviour* into an `Fprt`
//! table that [`fprt-exports`] installs as the DLL's C symbols.
//!
//! An engine is two traits: an [`Engine`] (process-wide state, a factory of
//! conductors) and a [`Conductor`] (one per-`ctx` session). The harness absorbs the
//! whole lifecycle around them — `library_initialize`, `conductor_start` (with the
//! decoded [`StartInfo`] config), the conductor state machine, the windowed
//! `sync_enter → event → pop* → sync_leave` turn, sleep/wake, and teardown via
//! `Drop`. For trivial/stateless engines, [`Server::from_event_fn`] takes a single
//! handler closure.
//!
//! A [`Server`] is static-free, so any number coexist for testing (drive them with
//! [`Server::initialize`]/[`start`](Server::start)/[`turn`](Server::turn)). The
//! process-wide singleton appears only at [`Server::into_process_engine`] (the DLL
//! export boundary), which returns the `Fprt` table for `fprt_exports::install`.
//!
//! [`Fprt`]: fprt_sys::Fprt
//! [`fprt-exports`]: https://docs.rs/fprt-exports

mod api;
mod command;
mod encoder;
mod engine;
mod event;
mod registry;
mod server;
mod session;

pub use api::{Conductor, Engine, InitError};
pub use encoder::{CallPool, Context, ENGINE_PANIC, Phase, phase};
pub use registry::Registry;
pub use server::{Emit, Sender, Server};
pub use session::{Outbox, Staged};

// The vocabulary an engine's signatures need, re-exported so an implementor depends
// on `fprt-server` for the harness and `fprt-core` only for the emit payloads.
pub use fprt_core::{Command, EngineError, Event, NextWake, StartInfo};
pub use fprt_sys::ctx::Ctx;
pub use fprt_sys::library_version::LibraryVersion;
