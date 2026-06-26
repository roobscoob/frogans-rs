//! `fprt-exports` — the DLL-side export shell.
//!
//! Turns a single runtime-installed [`Fprt`] table into the 179
//! `#[unsafe(no_mangle)]` C-ABI symbols the original `FrogansPlayer.exe` imports
//! by name. The whole implementor-facing surface is [`install`].
//!
//! ```ignore
//! fn build() -> fprt_exports::Fprt {
//!     fprt_exports::Fprt { /* …all 179 fields… */ }   // completeness gate
//! }
//! #[ctor::ctor]
//! fn boot() { let _ = fprt_exports::install(build()); }
//! ```
//!
//! `install` must run before the EXE calls any export — a load-time ctor /
//! `DllMain` is the natural place. Keep that light (it runs under the Windows
//! loader lock); do heavy startup (config, allocating engine state) in your
//! `library_initialize` impl, which the EXE calls at a safe point.
//!
//! Each export is a thin trampoline that forwards to the installed table. The UI
//! calls all share the 5-arg envelope, so their payload crosses as an opaque
//! `*mut/*const c_void` and is `.cast()` back to the field's payload type when
//! forwarded — which keeps the binding macros payload-type-free (see
//! `ui_exports.rs`). The 12 library/conductor calls have bespoke signatures and
//! are hand-written below.

use core::ffi::c_void;
use std::sync::OnceLock;

use fprt_sys::ctx::Ctx;
use fprt_sys::library_version::LibraryVersion;
use fprt_sys::mem::MempoolHandle;
use fprt_sys::start_information::StartInformation;
use fprt_sys::ustring::Ustring;

pub use fprt_sys::Fprt;

/// The installed table of implementations — write-once at startup, then read by
/// every export.
static FPRT: OnceLock<Fprt> = OnceLock::new();

/// Install the implementation table. Call once at startup, before the EXE invokes
/// any export. Returns the table back as `Err` if one was already installed.
pub fn install(table: Fprt) -> Result<(), Fprt> {
    FPRT.set(table)
}

/// Resolve the installed table. Panics (→ abort across the C boundary) if nothing
/// has been installed yet — that's a startup-ordering bug, not a runtime state.
#[inline]
fn table() -> &'static Fprt {
    FPRT.get()
        .expect("fprt-exports: no Fprt installed — call fprt_exports::install() at startup")
}

// ============================================================================
// Uniform UI trampolines (the 5-arg envelope; payload crosses opaquely).
// ============================================================================

/// A command export (`_pop`): payload is OUT (`*mut`).
macro_rules! export_pop {
    ($sym:ident, $field:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $sym(
            ctx: Ctx,
            payload: *mut c_void,
            status3: *mut i32,
            errbuf16: *mut Ustring,
            mempool_out: *mut MempoolHandle,
        ) -> i32 {
            unsafe { (table().$field)(ctx, payload.cast(), status3, errbuf16, mempool_out) }
        }
    };
}

/// An event export (`_report`): payload is IN (`*const`).
macro_rules! export_report {
    ($sym:ident, $field:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $sym(
            ctx: Ctx,
            payload: *const c_void,
            status3: *mut i32,
            errbuf16: *mut Ustring,
            mempool_out: *mut MempoolHandle,
        ) -> i32 {
            unsafe { (table().$field)(ctx, payload.cast(), status3, errbuf16, mempool_out) }
        }
    };
}

// ============================================================================
// Library + conductor trampolines (bespoke signatures).
// ============================================================================

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_library_initialize(
    version: *const LibraryVersion,
    status_lib_out: *mut u32,
) -> i32 {
    unsafe { (table().library_initialize)(version, status_lib_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_library_finalize(out_status: *mut u32) -> u32 {
    unsafe { (table().library_finalize)(out_status) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_library_is_initialized() -> bool {
    unsafe { (table().library_is_initialized)() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_library_report_allocated_arguments(
    out_count: *mut u32,
    out_total: *mut u32,
) -> i32 {
    unsafe { (table().library_report_allocated_arguments)(out_count, out_total) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_library_free_allocated_arguments(handle: MempoolHandle) -> bool {
    unsafe { (table().library_free_allocated_arguments)(handle) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_start(
    config: *const StartInformation,
    out_id: *mut Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe { (table().conductor_start)(config, out_id, status3, errbuf16, mempool_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_stop(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe { (table().conductor_stop)(ctx, status3, errbuf16, mempool_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_get_next_command(
    ctx: Ctx,
    has_command: *mut u32,
    command_id: *mut u32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe {
        (table().conductor_get_next_command)(
            ctx,
            has_command,
            command_id,
            status3,
            errbuf16,
            mempool_out,
        )
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_sync_enter(
    ctx: Ctx,
    elapsed_ms: i32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe { (table().conductor_sync_enter)(ctx, elapsed_ms, status3, errbuf16, mempool_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_sync_leave(
    ctx: Ctx,
    nextwake_out: *mut u32,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe { (table().conductor_sync_leave)(ctx, nextwake_out, status3, errbuf16, mempool_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_sleep_enter(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe { (table().conductor_sleep_enter)(ctx, status3, errbuf16, mempool_out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fprt_conductor_sleep_leave(
    ctx: Ctx,
    status3: *mut i32,
    errbuf16: *mut Ustring,
    mempool_out: *mut MempoolHandle,
) -> i32 {
    unsafe { (table().conductor_sleep_leave)(ctx, status3, errbuf16, mempool_out) }
}

// ============================================================================
// The 167 UI exports — the C-symbol ⇆ Fprt-field binding (the manifest).
// Generated from the Windows `fprt.dll` export table; one line per export.
// ============================================================================
include!("ui_exports.rs");
