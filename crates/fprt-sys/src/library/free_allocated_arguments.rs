//! `fprt_library_free_allocated_arguments` — release one argument buffer.
//!
//! Export ordinal 9 (VA `0x6d602500`).

use crate::mem::MempoolHandle;

/// `bool fprt_library_free_allocated_arguments(uint32_t handle);`
///
/// Releases the argument memory pool identified by `handle`, returning `true`
/// iff an entry was actually destroyed. Returns `false` — a harmless no-op —
/// when `handle` is [`MempoolHandle::EMPTY`], when the engine is not cruising,
/// or when the handle is stale / already freed. The host must call this exactly
/// once for every handle the engine hands out.
pub type FprtLibraryFreeAllocatedArguments =
    unsafe extern "C" fn(handle: MempoolHandle) -> bool;
