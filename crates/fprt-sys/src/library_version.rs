//! The `LibraryVersion` handshake type.

/// Read-only `major.minor.patch` version handshake passed to
/// `fprt_library_initialize` as `*const`.
///
/// The engine requires it to equal `0.36.16` exactly (else the initializer's
/// `BAD_VERSION`); see [`LibraryVersion::REQUIRED`]. ABI-identical to a
/// `*const i32` over three ints.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LibraryVersion {
    /// Major version. Required: `0`.
    pub major: i32,
    /// Minor version. Required: `36` (`0x24`).
    pub minor: i32,
    /// Patch version. Required: `16` (`0x10`).
    pub patch: i32,
}

impl LibraryVersion {
    /// The only version `fprt_library_initialize` accepts: `0.36.16`.
    pub const REQUIRED: Self = Self {
        major: 0,
        minor: 36,
        patch: 16,
    };
}
