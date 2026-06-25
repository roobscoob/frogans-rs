//! The `Ustring` string descriptor.

/// The universal 16-byte string descriptor the engine moves across the ABI.
///
/// UTF-8, **not** NUL-terminated — always use `len`. Written by
/// `_internal_fprt_expose_ustring` into a caller buffer (e.g. every conductor
/// call's `errbuf16` error sink) and read from input structs (e.g. the eight
/// strings inside the conductor config).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ustring {
    /// Byte length of the UTF-8 data at `utf8` (the engine writes an `i32`).
    pub len: i32,
    /// Pointer to `len` bytes of UTF-8 (not NUL-terminated). May be null.
    pub utf8: *const u8,
}
