//! The `ElementType` interactive-zone type (field 0 of every [`XButton`]).
//!
//! [`XButton`]: crate::ui::x_button::XButton

/// Sld interactive element-type code, base `0x6b724`. Resolved by the engine's
/// `expose_xbutton` from the button's kind; keys the cursor/tooltip and the
/// host's per-element class.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ElementType(pub u32);

impl ElementType {
    /// Un-typed base / default.
    pub const BASE: ElementType = ElementType(0x6b724);
    /// Entry zone, object not yet initialized.
    pub const ENTRY_UNINIT: ElementType = ElementType(0x6b725);
    /// Entry zone, initialized: text input.
    pub const ENTRY: ElementType = ElementType(0x6b726);
    /// Legal-information zone.
    pub const LEGAL: ElementType = ElementType(0x6b727);
    /// Link to another Frogans Site.
    pub const FROGANS_SITE: ElementType = ElementType(0x6b728);
    /// Way-out link, `http` scheme.
    pub const WAY_OUT_HTTP: ElementType = ElementType(0x6b729);
    /// Way-out link, `https` scheme.
    pub const WAY_OUT_HTTPS: ElementType = ElementType(0x6b72a);
    /// Way-out link, other scheme (e.g. `mailto`).
    pub const WAY_OUT_OTHER: ElementType = ElementType(0x6b72b);
    /// Clipboard zone, text variant.
    pub const CLIPBOARD_TEXT: ElementType = ElementType(0x6b72c);
    /// Clipboard zone, image variant.
    pub const CLIPBOARD_IMAGE: ElementType = ElementType(0x6b72d);
}
