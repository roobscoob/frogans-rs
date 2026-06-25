//! `fprt_library_report_allocated_arguments` — allocated-argument leak report.
//!
//! Export ordinal 12 (VA `0x6d602540`). Diagnostic only; not a 5-arg UI call.

/// `int32_t fprt_library_report_allocated_arguments(uint32_t *out_count, uint32_t *out_total);`
///
/// Writes the live argument-pool slot count to `out_count` and the summed byte
/// total to `out_total`. Returns `1` on success, or `0` when the engine is not
/// cruising or either pointer is NULL. Each non-NULL out is zeroed before work,
/// and either pointer may be NULL.
pub type FprtLibraryReportAllocatedArguments =
    unsafe extern "C" fn(out_count: *mut u32, out_total: *mut u32) -> i32;
