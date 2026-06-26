//! The front door: [`Library`], the safe entry point to the initialized engine.

use std::marker::PhantomData;
use std::sync::Arc;

use fprt_sys::ctx::Ctx;
use fprt_sys::library_version::LibraryVersion;
use fprt_sys::mem::MempoolHandle;

use crate::call::{check, empty_errbuf};
use crate::conductor::{Conductor, ConductorConfig};
use crate::engine::{self, EngineInner};
use crate::error::EngineError;
use crate::host::FprtHost;

/// The live, initialized engine — the safe entry point to everything else.
///
/// Holding a `Library` *is the proof* that the engine is cruising: there is no
/// "loaded but uninitialized" state to misuse. Construction runs
/// `fprt_library_initialize`; the engine `finalize`s when the last handle to it
/// drops — and that includes any [`Pooled`](crate::Pooled) value or
/// [`EngineError`](crate::EngineError) still holding engine-owned data, so such
/// data is always safe to read for as long as you keep it.
///
/// The host is type-erased ([`Box<dyn FprtHost>`], inside the shared
/// [`EngineInner`]) so that everything built on top of `Library` — conductors,
/// and eventually the UI calls — stays a plain concrete type instead of being
/// generic over the host.
///
/// `Library` is `!Send + !Sync`: it is the home of the engine's single-thread
/// API and must stay on the thread it was created on. Thread-safe capabilities
/// are reached not through `Library` itself but through the `Arc<EngineInner>`
/// it shares into handles (pools, errors, and — where the operation is
/// thread-safe — conductors), which *are* `Send + Sync`.
pub struct Library {
    engine: Arc<EngineInner>,
    /// Opts `Library` out of `Send`/`Sync`. `Arc<EngineInner>` is thread-safe on
    /// its own, so without this marker `Library` would auto-derive both.
    _single_thread: PhantomData<*const ()>,
}

/// The live argument-pool census from [`Library::allocated_arguments`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocReport {
    /// Number of live argument-pool slots.
    pub count: u32,
    /// Summed byte total across those slots.
    pub total: u32,
}

impl Library {
    /// Initialize the engine through a caller-supplied host.
    ///
    /// Safe despite calling into C: the `unsafe` obligation lives on
    /// [`FprtHost`] (the host promises a valid, correctly-typed table), so the
    /// only ways this fails are genuine engine states — surfaced as an
    /// [`EngineError`] whose [`code`](EngineError::code) you can compare against
    /// the `fprt_sys::library::initialize` constants (e.g. `BAD_VERSION` when
    /// `version` isn't [`LibraryVersion::REQUIRED`]). Init carries no message, so
    /// the error's `message` is always `None`. Initializing an engine that is
    /// already cruising returns `ALREADY_INITIALIZED` rather than a second
    /// handle — the engine, not this wrapper, enforces single ownership.
    pub fn initialize<H: FprtHost + 'static>(
        host: H,
        version: LibraryVersion,
    ) -> Result<Library, EngineError> {
        let host: Box<dyn FprtHost> = Box::new(host);

        let mut status: u32 = 0;
        let ok = {
            // Serialize this transition against any concurrent init/finalize.
            let _guard = engine::lifecycle_lock();
            // SAFETY: `host` upholds the `FprtHost` contract — the table is valid
            // and stays valid for this call. `version` and `status` are valid ptrs.
            unsafe { (host.methods().library_initialize)(&version, &mut status) }
        };

        if ok == 1 {
            Ok(Library {
                engine: Arc::new(EngineInner::new(host)),
                _single_thread: PhantomData,
            })
        } else {
            // Covers the already-cruising case (`ALREADY_INITIALIZED`) too: we
            // never built an `EngineInner`, so nothing finalizes and no engine
            // we don't own is torn down.
            Err(EngineError::new(status as i32, None))
        }
    }

    /// Convenience constructor: load the engine module at `path` and initialize.
    #[cfg(feature = "libloading")]
    pub fn open(
        path: impl AsRef<std::ffi::OsStr>,
        version: LibraryVersion,
    ) -> Result<Library, OpenError> {
        let host = crate::host::LibloadingHost::open(path).map_err(OpenError::Load)?;
        Library::initialize(host, version).map_err(OpenError::Init)
    }

    /// The engine's live argument-pool census (a leak diagnostic), or `None` if
    /// the engine declined to report (not cruising / internal refusal).
    pub fn allocated_arguments(&self) -> Option<AllocReport> {
        let (mut count, mut total) = (0u32, 0u32);
        // SAFETY: table valid for the call; both out-pointers are valid.
        let ok = unsafe {
            (self.engine.methods().library_report_allocated_arguments)(&mut count, &mut total)
        };
        (ok == 1).then_some(AllocReport { count, total })
    }

    /// Boot a conductor from a [`ConductorConfig`].
    ///
    /// The conductor is created on this — the library's — thread, which is where
    /// its `!Send` main-thread API stays.
    pub fn spawn_conductor(&self, config: ConductorConfig<'_>) -> Result<Conductor, EngineError> {
        // `raw`'s `Ustring`s borrow `config`'s strings; both outlive the call.
        let raw = config.to_raw();
        let mut out_id = Ctx(0);
        let mut status = 0i32;
        let mut errbuf = empty_errbuf();
        let mut mempool = MempoolHandle::EMPTY;
        // SAFETY: valid config + out pointers; table valid via the live engine.
        unsafe {
            (self.engine.methods().conductor_start)(
                &raw,
                &mut out_id,
                &mut status,
                &mut errbuf,
                &mut mempool,
            );
        }
        check(&self.engine, status, errbuf, mempool)?;
        Ok(Conductor::new(out_id, Arc::clone(&self.engine)))
    }
}

/// Failure from the turnkey [`Library::open`] path: either the module wouldn't
/// load/resolve, or it loaded but the engine wouldn't initialize.
#[cfg(feature = "libloading")]
#[derive(Debug)]
pub enum OpenError {
    /// The module failed to load or a symbol failed to resolve.
    Load(libloading::Error),
    /// The module loaded but `fprt_library_initialize` failed.
    Init(EngineError),
}

#[cfg(feature = "libloading")]
impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OpenError::Load(e) => write!(f, "failed to load FPRT engine module: {e}"),
            OpenError::Init(e) => write!(f, "{e}"),
        }
    }
}

#[cfg(feature = "libloading")]
impl std::error::Error for OpenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OpenError::Load(e) => Some(e),
            OpenError::Init(e) => Some(e),
        }
    }
}
