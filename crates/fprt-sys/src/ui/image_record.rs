//! The `ImageRecord` encoded-image descriptor (`0x18`).

/// A `{ width, height }` view of an [`ImageRecord`]'s first 8 bytes — the host's
/// reading of the engine's handle.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

/// The first 8 bytes of an [`ImageRecord`]: the engine writes an opaque `u64`
/// bmp-rgba handle; the host reads it as an [`ImageSize`].
#[repr(C)]
#[derive(Clone, Copy)]
pub union ImageDim {
    pub fmt_handle: u64,
    pub size: ImageSize,
}

/// A mempool-backed encoded (PNG) image the engine hands the host (`0x18`).
///
/// `buffer` is mempool-owned — free it via `fprt_library_free_allocated_arguments`
/// with the call's `mempool_out` token after consuming the bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageRecord {
    /// Engine handle / host `{ width, height }` (see [`ImageDim`]).
    pub dim: ImageDim,
    /// Encoded byte length.
    pub byte_len: u32,
    // +0x0c: 4 bytes implicit padding → buffer aligns to +0x10.
    /// Mempool-allocated encoded image bytes.
    pub buffer: *mut u8,
}
