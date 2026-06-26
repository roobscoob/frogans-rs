//! Safe mirrors of the engine's visual X-types — shared by [`menu`](super::menu)
//! and [`sitehandler`](super::sitehandler).
//!
//! Resolved geometry gets named fields: the [`SldRect`] screen rect, and the
//! `[x, y, width, height]` slide rects ([`Rect`]) carried by [`Piece`]s and
//! [`Rollover`]s. The words whose meaning is still unresolved — the two trailing
//! words of [`Representation::geom`], `entry_state`, the opaque `raw_handle` —
//! pass through verbatim as integers, to be pinned down against real render data.
//! Images stay
//! format-agnostic [`PooledImage`]s; where a node carries more than one (an
//! [`XPiece`]'s raw plane + encoded PNG) the *field names* carry the distinction.

use fprt_sys::ui::sld_rect::SldRect;
use fprt_sys::ui::x_button::XButton;
use fprt_sys::ui::x_piece::XPiece;
use fprt_sys::ui::x_representation::XRepresentation;
use fprt_sys::ui::x_rollover::XRollover;
use fprt_sys::ui::ElementType as RawElementType;

use crate::pool::{Pool, PooledImage, PooledString};

/// A screen rectangle in FPRT top-left coordinates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScreenRect {
    /// Index into the host screen list; out-of-range ⇒ primary screen.
    pub screen_index: i32,
    /// X offset within the screen's frame.
    pub x: i32,
    /// Y offset (FPRT top-left).
    pub y: i32,
}

/// A rectangle in slide pixel space: top-left origin plus size.
///
/// Resolved from the engine's four geometry words `[x, y, width, height]` —
/// confirmed against real render data, where `width`/`height` match the attached
/// image's dimensions exactly.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    /// X offset of the top-left corner.
    pub x: i32,
    /// Y offset of the top-left corner.
    pub y: i32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
}

impl Rect {
    fn from_raw(raw: [i32; 4]) -> Self {
        Rect {
            x: raw[0],
            y: raw[1],
            width: raw[2],
            height: raw[3],
        }
    }
}

impl ScreenRect {
    pub(crate) fn from_raw(raw: SldRect) -> Self {
        ScreenRect {
            screen_index: raw.screen_index,
            x: raw.x,
            y: raw.y,
        }
    }

    /// `present_flag == 0` ⇒ `None` (no rect supplied; host centers/ignores).
    pub(crate) fn option(present_flag: u32, raw: SldRect) -> Option<Self> {
        (present_flag != 0).then(|| ScreenRect::from_raw(raw))
    }
}

/// The interactive-zone type of a [`Button`] — keys the cursor/tooltip and the
/// host's per-element behaviour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ElementType {
    /// Un-typed base / default.
    Base,
    /// Entry zone, not yet initialized.
    EntryUninit,
    /// Entry zone: text input.
    Entry,
    /// Legal-information zone.
    Legal,
    /// Link to another Frogans Site.
    FrogansSite,
    /// Way-out link, `http`.
    WayOutHttp,
    /// Way-out link, `https`.
    WayOutHttps,
    /// Way-out link, other scheme (e.g. `mailto`).
    WayOutOther,
    /// Clipboard zone, text variant.
    ClipboardText,
    /// Clipboard zone, image variant.
    ClipboardImage,
    /// An engine value outside the documented set.
    Other(u32),
}

impl ElementType {
    pub(crate) fn from_raw(raw: RawElementType) -> Self {
        match raw {
            RawElementType::BASE => ElementType::Base,
            RawElementType::ENTRY_UNINIT => ElementType::EntryUninit,
            RawElementType::ENTRY => ElementType::Entry,
            RawElementType::LEGAL => ElementType::Legal,
            RawElementType::FROGANS_SITE => ElementType::FrogansSite,
            RawElementType::WAY_OUT_HTTP => ElementType::WayOutHttp,
            RawElementType::WAY_OUT_HTTPS => ElementType::WayOutHttps,
            RawElementType::WAY_OUT_OTHER => ElementType::WayOutOther,
            RawElementType::CLIPBOARD_TEXT => ElementType::ClipboardText,
            RawElementType::CLIPBOARD_IMAGE => ElementType::ClipboardImage,
            _ => ElementType::Other(raw.0),
        }
    }
}

/// One image fragment of a [`Rollover`] — a placement, a raw RGBA plane, and an
/// encoded PNG.
#[derive(Debug)]
pub struct Piece {
    /// Placement rectangle (top-left origin + size).
    pub geom: Rect,
    /// Raw-mode code (`0x838 - rawmode` / `2000` / `0x67` = empty — passthrough).
    pub kind: i32,
    /// The raw RGBA plane.
    pub raw: Option<PooledImage>,
    /// The encoded PNG image.
    pub encoded: Option<PooledImage>,
}

impl Piece {
    /// # Safety
    /// `p`'s image records must point into `pool`.
    unsafe fn from_raw(p: XPiece, pool: &Pool) -> Self {
        unsafe {
            Piece {
                geom: Rect::from_raw(p.geom),
                kind: p.kind,
                raw: pool.image(p.plane),
                encoded: pool.image(p.image),
            }
        }
    }
}

/// One interactive rollover region of a [`Representation`].
#[derive(Debug)]
pub struct Rollover {
    /// Clickable rectangle (top-left origin + size).
    pub region: Rect,
    /// The per-state image pieces.
    pub pieces: Vec<Piece>,
}

impl Rollover {
    /// # Safety
    /// `r.pieces` must point at `r.piece_count` records in `pool`.
    unsafe fn from_raw(r: XRollover, pool: &Pool) -> Self {
        let mut pieces = Vec::with_capacity(r.piece_count.max(0) as usize);
        if !r.pieces.is_null() {
            for i in 0..r.piece_count.max(0) as usize {
                pieces.push(unsafe { Piece::from_raw(*r.pieces.add(i), pool) });
            }
        }
        Rollover {
            region: Rect::from_raw(r.region),
            pieces,
        }
    }
}

/// One rendered slide layer: an RGBA image plus its interactive rollover regions.
#[derive(Debug)]
pub struct Representation {
    /// Opaque engine dimensions/format handle (passthrough).
    pub raw_handle: u64,
    /// Rendered RGBA slide pixels.
    pub image: Option<PooledImage>,
    /// Six geometry words: origin `[0..2]` + size `[2..4]` + two unresolved
    /// `[4..6]` (passthrough).
    pub geom: [i32; 6],
    /// The slide's interactive rollover regions.
    pub rollovers: Vec<Rollover>,
}

impl Representation {
    /// # Safety
    /// `rep`'s image + `rep.rollovers` array must point into `pool`.
    pub(crate) unsafe fn from_raw(rep: XRepresentation, pool: &Pool) -> Self {
        let mut rollovers = Vec::with_capacity(rep.rollover_count.max(0) as usize);
        if !rep.rollovers.is_null() {
            for i in 0..rep.rollover_count.max(0) as usize {
                rollovers.push(unsafe { Rollover::from_raw(*rep.rollovers.add(i), pool) });
            }
        }
        Representation {
            raw_handle: rep.raw_handle,
            image: unsafe { pool.image(rep.image) },
            geom: rep.geom,
            rollovers,
        }
    }
}

/// One interactive zone element of an `update_visual` button array.
#[derive(Debug)]
pub struct Button {
    /// The zone type.
    pub element_type: ElementType,
    /// Entry / address text.
    pub label: Option<PooledString>,
    /// `true` ⇒ entry text concealed (password-style).
    pub concealed: bool,
    /// Entry common-data state (exact enum unresolved — passthrough).
    pub entry_state: i32,
    /// `frogans_site` icon image (populated only for [`ElementType::FrogansSite`]).
    pub icon: Option<PooledImage>,
}

impl Button {
    /// # Safety
    /// `b`'s label + icon must point into `pool`.
    unsafe fn from_raw(b: XButton, pool: &Pool) -> Self {
        unsafe {
            Button {
                element_type: ElementType::from_raw(b.element_type),
                label: pool.string(b.label),
                concealed: b.concealed == 1,
                entry_state: b.entry_state,
                icon: pool.image(b.icon_image),
            }
        }
    }

    /// Build the safe button list from a `*mut XButton` array of `count` entries.
    ///
    /// # Safety
    /// `ptr` must point at `count` records in `pool` (or be null).
    pub(crate) unsafe fn list(ptr: *mut XButton, count: usize, pool: &Pool) -> Vec<Button> {
        let mut out = Vec::with_capacity(count);
        if !ptr.is_null() {
            for i in 0..count {
                out.push(unsafe { Button::from_raw(*ptr.add(i), pool) });
            }
        }
        out
    }
}
