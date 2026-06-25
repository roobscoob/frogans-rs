//! `fprt_library_finalize` — reverse teardown of the library.
//!
//! Export ordinal 8 (VA `0x6d602480`).

/// `out_status`: library was not initialized — no-op.
pub const NOT_INITIALIZED: u32 = 0x0;
/// `out_status`: initialized but fleet not cruising.
pub const NOT_CRUISING: u32 = 0xc9;

/// `uint32_t fprt_library_finalize(uint32_t *out_status);`
///
/// Returns `1` on success — leaving `out_status` untouched — or `0` otherwise.
/// A NULL `out_status` is a no-op returning `0`.
pub type FprtLibraryFinalize = unsafe extern "C" fn(out_status: *mut u32) -> u32;
