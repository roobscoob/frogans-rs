//! The `MempoolHandle` argument-buffer handle.

/// An opaque handle to an engine-allocated argument memory pool — **not a
/// pointer**, a packed 32-bit token.
///
/// The engine owns every argument buffer it writes across the ABI (the
/// `{len, utf8}` strings, image blobs, diagnostic strings) and hands the host
/// these cookies instead of raw pointers. The host must release each one with
/// `fprt_library_free_allocated_arguments`.
///
/// Forgery-resistant: the low 24 bits are a slot index XOR a per-run key, and
/// the full value is re-checked against the slot's stored key, so stale or
/// forged handles are rejected rather than acted on.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MempoolHandle(pub u32);

impl MempoolHandle {
    pub const EMPTY: MempoolHandle = MempoolHandle(0x3b9acde8);
}
