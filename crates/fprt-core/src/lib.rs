//! `fprt-core` — the memory primitives shared by both sides of the FPRT ABI.
//!
//! Every argument the engine and host exchange — strings, image blobs,
//! diagnostic messages — lives in a *pool* the producing side owns and the
//! consuming side reads. This crate models that pool once, neutrally, so the
//! caller wrapper (`fprt`/client) and the implementor wrapper (server) can both
//! build on it instead of re-deriving it:
//!
//!   * [`Pooled<T>`](pool::Pooled) — a `*const T` into pool memory plus a
//!     refcounted [`Pool`](pool::Pool) keeping it alive; the typed views
//!     [`PooledString`](pool::PooledString) / [`PooledImage`](pool::PooledImage)
//!     sit on top.
//!   * [`Anchor`](pool::Anchor) — what a `Pool` is *backed by*. The client backs
//!     it with a foreign engine mempool handle (freed over FFI); the server backs
//!     it with an owned [`Arena`](arena::Arena). `Pooled` is identical either way.
//!   * [`Arena`](arena::Arena) / [`OwnedPool`](pool::OwnedPool) — the owned,
//!     `Sync`, append-only allocator: copy data in, get a stable `Pooled` back.
//!     Usable freestanding, with no engine or DLL in sight.

pub mod arena;
pub mod command;
pub mod component;
pub mod error;
pub mod event;
pub mod nextwake;
pub mod pool;
pub mod start_info;
pub mod wire;

pub use command::Command;
pub use error::EngineError;
pub use event::Event;
pub use nextwake::NextWake;
pub use start_info::StartInfo;
