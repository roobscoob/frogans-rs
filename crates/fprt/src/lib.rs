//! `fprt` — a safe wrapper around the raw [`fprt_sys`] Frogans Player C ABI.
//!
//! The raw crate is a behaviour-free contract: `#[repr(C)]` types, status codes,
//! and the [`Fprt`](fprt_sys::Fprt) table of function pointers. This crate adds
//! the *behaviour* that makes those exports safe to call:
//!
//!   * the **front door** ([`Library`]) — owns the engine's process-global
//!     "fleet cruising" state: constructing one runs `fprt_library_initialize`,
//!     dropping one runs `fprt_library_finalize`, and a process-wide guard keeps
//!     it a singleton;
//!   * the **host seam** ([`FprtHost`]) — the one thing a `Library` needs: a
//!     source of a live [`Fprt`] table whose pointers stay valid for as long as
//!     the host does. `libloading` is just one implementation
//!     ([`LibloadingHost`], behind the `libloading` feature); a static-link host
//!     would be another.
//!
//! What this crate deliberately does *not* model with Rust lifetimes: the
//! engine's `Ctx` and `MempoolHandle` are self-validating integer tokens, not
//! borrows — misusing them yields an error code, never UB — so they will be
//! owned `Copy` values, not lifetime-bound references. The only genuine borrows
//! are the DLL mapping (encoded by [`FprtHost::methods`] returning `&Fprt`) and
//! engine-owned string buffers (handled later, by the call layer).

mod arena;
mod call;
mod conductor;
mod engine;
mod error;
mod host;
mod library;
mod pool;

pub use conductor::component::{
    application, blocked, devtools, favorites, inputfa, inspector, language, leaptofrogans,
    legalinformation, menu, pad, recentlyvisited, recovery, sitehandler, update, visual, zoom,
};
pub use conductor::{
    Command, CommandError, Commands, Conductor, ConductorConfig, DeploymentMode, Directories,
    ImageFormat, Manifest, Nature, NextWake, Report, Turn,
};
pub use error::EngineError;
pub use host::FprtHost;
pub use library::{AllocReport, Library};
pub use pool::{Pooled, PooledImage, PooledString};

#[cfg(feature = "libloading")]
pub use library::OpenError;

#[cfg(feature = "libloading")]
pub use host::LibloadingHost;

// Re-export the raw table so downstream `impl FprtHost` blocks need only one dep.
pub use fprt_sys::Fprt;

// Re-export the version handshake type, since callers must pass it to initialize.
pub use fprt_sys::library_version::LibraryVersion;
