//! `fprt_library_initialize` — the library entry-point initializer.
//!
//! Export ordinal 10 (VA `0x6d602370`). Not one of the uniform 5-arg UI calls.

use crate::library_version::LibraryVersion;

/// `status_lib_out`: already initialized — idempotent, not an error.
pub const ALREADY_INITIALIZED: u32 = 0x3b9a_ca01;
/// `status_lib_out`: version struct was not `{0, 0x24, 0x10}`.
pub const BAD_VERSION: u32 = 0x3b9a_ca02;
/// `status_lib_out`: fleet already cruising (inconsistent state).
pub const FLEET_CRUISING: u32 = 0x3b9a_cac9;
/// `status_lib_out`: subsystem takeoff failed.
pub const INIT_FAILED: u32 = 0x3b9a_caca;

/// `int32_t fprt_library_initialize(const int32_t *version, uint32_t *status_lib_out);`
///
/// Returns `1` on success — leaving `status_lib_out` untouched — or `0` otherwise,
/// with one of the status codes above written to `status_lib_out`. A NULL
/// `status_lib_out` is a no-op returning `0`.
pub type FprtLibraryInitialize =
    unsafe extern "C" fn(version: *const LibraryVersion, status_lib_out: *mut u32) -> i32;
