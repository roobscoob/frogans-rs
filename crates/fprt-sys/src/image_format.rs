//! The `ImageFormat` pixel-format selector.

/// Pixel format for images the engine encodes and hands to the host.
///
/// In the conductor config, one field selects the standalone-image format and
/// another the xrepresentation/site format. (Windows ships
/// [`ImageFormat::BGRA_PREMULTIPLIED`], macOS [`ImageFormat::RGBA_PREMULTIPLIED`].)
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImageFormat(pub u32);

impl ImageFormat {
    /// PNG-encoded.
    pub const PNG: ImageFormat = ImageFormat(0x2dd27a);
    /// Raw RGBA, straight alpha.
    pub const RGBA: ImageFormat = ImageFormat(0x2dd27b);
    /// Raw ABGR, straight alpha.
    pub const ABGR: ImageFormat = ImageFormat(0x2dd27c);
    /// Raw ARGB, straight alpha.
    pub const ARGB: ImageFormat = ImageFormat(0x2dd27d);
    /// Raw BGRA, straight alpha.
    pub const BGRA: ImageFormat = ImageFormat(0x2dd27e);
    /// Raw RGBA, premultiplied alpha.
    pub const RGBA_PREMULTIPLIED: ImageFormat = ImageFormat(0x2dd27f);
    /// Raw ABGR, premultiplied alpha.
    pub const ABGR_PREMULTIPLIED: ImageFormat = ImageFormat(0x2dd280);
    /// Raw ARGB, premultiplied alpha.
    pub const ARGB_PREMULTIPLIED: ImageFormat = ImageFormat(0x2dd281);
    /// Raw BGRA, premultiplied alpha.
    pub const BGRA_PREMULTIPLIED: ImageFormat = ImageFormat(0x2dd282);
}
