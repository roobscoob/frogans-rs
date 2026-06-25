//! `fprt_library_is_initialized` — the library-init predicate.
//!
//! Export ordinal 11 (VA `0x6d6024f0`). Read-only; returns the cruising flag.

/// `bool fprt_library_is_initialized(void);`
///
/// Returns `true` iff `fprt_library_initialize` has succeeded and
/// `fprt_library_finalize` has not yet run. No parameters; touches no state.
pub type FprtLibraryIsInitialized = unsafe extern "C" fn() -> bool;
