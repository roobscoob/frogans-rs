//! Safe mirrors of the engine's visual X-types — shared by `menu` and
//! `sitehandler`.
//!
//! Resolved geometry gets named fields: the [`ScreenRect`] screen rect, and the
//! `[x, y, width, height]` slide rects ([`Rect`]) carried by [`Piece`]s and
//! [`Rollover`]s. The words whose meaning is still unresolved pass through
//! verbatim as integers. Images stay format-agnostic [`PooledImage`]s.
//!
//! Both directions: `from_raw` (the engine produces visual data, the host reads
//! it) and `to_raw` (the server side builds the X-type arrays into a pool — the
//! faithful inverse, gated by the round-trip tests).

use fprt_sys::ui::ElementType as RawElementType;
use fprt_sys::ui::sld_rect::SldRect;
use fprt_sys::ui::x_button::XButton;
use fprt_sys::ui::x_piece::XPiece;
use fprt_sys::ui::x_representation::XRepresentation;
use fprt_sys::ui::x_rollover::XRollover;

use crate::pool::{OwnedPool, Pool, PooledImage, PooledString};
use crate::wire::{image_record, ustring_opt};

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

    fn to_raw(self) -> [i32; 4] {
        [self.x, self.y, self.width, self.height]
    }
}

impl ScreenRect {
    /// Map the raw screen rect.
    pub fn from_raw(raw: SldRect) -> Self {
        ScreenRect {
            screen_index: raw.screen_index,
            x: raw.x,
            y: raw.y,
        }
    }

    /// `present_flag == 0` ⇒ `None` (no rect supplied; host centers/ignores).
    pub fn option(present_flag: u32, raw: SldRect) -> Option<Self> {
        (present_flag != 0).then(|| ScreenRect::from_raw(raw))
    }
}

/// The interactive-zone type of a [`Button`].
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
    /// Map the raw element type.
    pub fn from_raw(raw: RawElementType) -> Self {
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

    /// Map back to the raw element type.
    pub fn to_raw(self) -> RawElementType {
        match self {
            ElementType::Base => RawElementType::BASE,
            ElementType::EntryUninit => RawElementType::ENTRY_UNINIT,
            ElementType::Entry => RawElementType::ENTRY,
            ElementType::Legal => RawElementType::LEGAL,
            ElementType::FrogansSite => RawElementType::FROGANS_SITE,
            ElementType::WayOutHttp => RawElementType::WAY_OUT_HTTP,
            ElementType::WayOutHttps => RawElementType::WAY_OUT_HTTPS,
            ElementType::WayOutOther => RawElementType::WAY_OUT_OTHER,
            ElementType::ClipboardText => RawElementType::CLIPBOARD_TEXT,
            ElementType::ClipboardImage => RawElementType::CLIPBOARD_IMAGE,
            ElementType::Other(v) => RawElementType(v),
        }
    }
}

/// One image fragment of a [`Rollover`] — a placement, a raw RGBA plane, and an
/// encoded PNG.
#[derive(Debug)]
pub struct Piece {
    /// Placement rectangle (top-left origin + size).
    pub geom: Rect,
    /// Raw-mode code (passthrough).
    pub kind: i32,
    /// The raw RGBA plane.
    pub raw: Option<PooledImage>,
    /// The encoded PNG image.
    pub encoded: Option<PooledImage>,
}

impl Piece {
    /// Build a piece, allocating the raw plane and encoded image into `pool`.
    ///
    /// `plane`/`encoded` are each `(bytes, width, height)`; pass `None` for the
    /// 2001 (`enc=None`) hit-silhouette case.
    pub fn new(
        pool: &OwnedPool,
        geom: Rect,
        kind: i32,
        plane: Option<(&[u8], u32, u32)>,
        encoded: Option<(&[u8], u32, u32)>,
    ) -> Self {
        Piece {
            geom,
            kind,
            raw: plane.map(|(b, w, h)| pool.alloc_image(b, w, h)),
            encoded: encoded.map(|(b, w, h)| pool.alloc_image(b, w, h)),
        }
    }

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

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Piece {
            geom: self.geom,
            kind: self.kind,
            raw: pool.clone_image_opt(&self.raw),
            encoded: pool.clone_image_opt(&self.encoded),
        }
    }

    /// Encode into the raw piece, pointing its image records at this piece's
    /// pooled bytes.
    fn to_raw(&self, _pool: &OwnedPool) -> XPiece {
        XPiece {
            geom: self.geom.to_raw(),
            kind: self.kind,
            plane: image_record(self.raw.as_ref()),
            image: image_record(self.encoded.as_ref()),
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
    /// A rollover region carrying `pieces` (in paint order).
    pub fn new(region: Rect, pieces: Vec<Piece>) -> Self {
        Rollover { region, pieces }
    }

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

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Rollover {
            region: self.region,
            pieces: self.pieces.iter().map(|p| p.copy_into(pool)).collect(),
        }
    }

    /// Encode into the raw rollover, allocating its piece array into `pool`.
    fn to_raw(&self, pool: &OwnedPool) -> XRollover {
        let (piece_count, pieces) = if self.pieces.is_empty() {
            (0, core::ptr::null_mut())
        } else {
            let descriptors: Vec<XPiece> = self.pieces.iter().map(|p| p.to_raw(pool)).collect();
            (
                self.pieces.len() as i32,
                pool.alloc_slice(&descriptors).cast::<XPiece>() as *mut XPiece,
            )
        };
        XRollover {
            region: self.region.to_raw(),
            piece_count,
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
    /// Build a representation, allocating its resting image into `pool`.
    /// `image` is `(bytes, width, height)`; pass `None` for no image.
    pub fn new(
        pool: &OwnedPool,
        raw_handle: u64,
        image: Option<(&[u8], u32, u32)>,
        geom: [i32; 6],
        rollovers: Vec<Rollover>,
    ) -> Self {
        Representation {
            raw_handle,
            image: image.map(|(b, w, h)| pool.alloc_image(b, w, h)),
            geom,
            rollovers,
        }
    }

    /// # Safety
    /// `rep`'s image + `rep.rollovers` array must point into `pool`.
    pub unsafe fn from_raw(rep: XRepresentation, pool: &Pool) -> Self {
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

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Representation {
            raw_handle: self.raw_handle,
            image: pool.clone_image_opt(&self.image),
            geom: self.geom,
            rollovers: self.rollovers.iter().map(|r| r.copy_into(pool)).collect(),
        }
    }

    /// Encode into the raw representation, allocating its image + rollover array
    /// into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> XRepresentation {
        let (rollover_count, rollovers) = if self.rollovers.is_empty() {
            (0, core::ptr::null_mut())
        } else {
            let descriptors: Vec<XRollover> =
                self.rollovers.iter().map(|r| r.to_raw(pool)).collect();
            (
                self.rollovers.len() as i32,
                pool.alloc_slice(&descriptors).cast::<XRollover>() as *mut XRollover,
            )
        };
        XRepresentation {
            raw_handle: self.raw_handle,
            image: image_record(self.image.as_ref()),
            geom: self.geom,
            rollover_count,
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
    /// Build a button, allocating its label into `pool`. `icon` is each
    /// `(bytes, width, height)`; pass `None` outside [`ElementType::FrogansSite`].
    pub fn new(
        pool: &OwnedPool,
        element_type: ElementType,
        label: Option<&str>,
        concealed: bool,
        entry_state: i32,
        icon: Option<(&[u8], u32, u32)>,
    ) -> Self {
        Button {
            element_type,
            label: label.map(|s| pool.alloc_str(s)),
            concealed,
            entry_state,
            icon: icon.map(|(b, w, h)| pool.alloc_image(b, w, h)),
        }
    }

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
    pub unsafe fn list(ptr: *mut XButton, count: usize, pool: &Pool) -> Vec<Button> {
        let mut out = Vec::with_capacity(count);
        if !ptr.is_null() {
            for i in 0..count {
                out.push(unsafe { Button::from_raw(*ptr.add(i), pool) });
            }
        }
        out
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Button {
            element_type: self.element_type,
            label: pool.clone_str_opt(&self.label),
            concealed: self.concealed,
            entry_state: self.entry_state,
            icon: pool.clone_image_opt(&self.icon),
        }
    }

    /// Encode into the raw button, pointing its label/icon at this button's
    /// pooled bytes.
    fn to_raw(&self, _pool: &OwnedPool) -> XButton {
        XButton {
            element_type: self.element_type.to_raw(),
            label: ustring_opt(self.label.as_ref()),
            concealed: u32::from(self.concealed),
            entry_state: self.entry_state,
            icon_image: image_record(self.icon.as_ref()),
        }
    }

    /// Encode a button list into a `*mut XButton` array in `pool`, returning the
    /// `(count, ptr)` pair the payloads expect (`(0, null)` when empty).
    pub fn list_to_raw(buttons: &[Button], pool: &OwnedPool) -> (usize, *mut XButton) {
        if buttons.is_empty() {
            return (0, core::ptr::null_mut());
        }
        let descriptors: Vec<XButton> = buttons.iter().map(|b| b.to_raw(pool)).collect();
        (
            buttons.len(),
            pool.alloc_slice(&descriptors).cast::<XButton>() as *mut XButton,
        )
    }
}

/// Test-only builders for a representative visual value, shared by the round-trip
/// tests in `visual`, `menu::cmd_update_visual`, and `sitehandler::cmd_update_visual`.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// A representation with a 1×1 resting image and one rollover zone carrying a
    /// `2001` hit-silhouette piece (plane only, `enc=None`) followed by a `2002`
    /// base layer (plane + encoded image) — paint order preserved.
    pub(crate) fn sample_representation(pool: &OwnedPool) -> Representation {
        // 1-bpp plane for a 2×3 zone: ceil(2*3/8) = 1 byte.
        let plane_2001: &[u8] = &[0b1011_0100];
        let plane_2002: &[u8] = &[0b0010_1101];
        let png: &[u8] = &[0x89, b'P', b'N', b'G', 1, 2, 3, 4];
        let rest: &[u8] = &[0xde, 0xad, 0xbe, 0xef];

        let piece_2001 = Piece::new(
            pool,
            Rect { x: 0, y: 0, width: 2, height: 3 },
            0x67,
            Some((plane_2001, 2, 3)),
            None,
        );
        let piece_2002 = Piece::new(
            pool,
            Rect { x: 0, y: 0, width: 2, height: 3 },
            0x66,
            Some((plane_2002, 2, 3)),
            Some((png, 1, 1)),
        );
        let rollover = Rollover::new(
            Rect { x: 4, y: 5, width: 2, height: 3 },
            vec![piece_2001, piece_2002],
        );
        Representation::new(
            pool,
            0x1234_5678_9abc_def0,
            Some((rest, 1, 1)),
            [10, 20, 30, 40, 50, 60],
            vec![rollover],
        )
    }

    /// Two representative buttons: a concealed entry and a frogans-site link with
    /// an icon image.
    pub(crate) fn sample_buttons(pool: &OwnedPool) -> Vec<Button> {
        let icon: &[u8] = &[1, 1, 2, 3, 5, 8];
        vec![
            Button::new(pool, ElementType::Entry, Some("réseau"), true, 7, None),
            Button::new(
                pool,
                ElementType::FrogansSite,
                Some("frogans*example"),
                false,
                0,
                Some((icon, 1, 1)),
            ),
        ]
    }

    /// Assert `back` round-tripped from [`sample_representation`].
    pub(crate) fn assert_representation(back: &Representation) {
        assert_eq!(back.raw_handle, 0x1234_5678_9abc_def0);
        assert_eq!(back.geom, [10, 20, 30, 40, 50, 60]);
        let img = back.image.as_ref().expect("resting image present");
        assert_eq!(img.bytes(), &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!((img.width(), img.height()), (1, 1));

        assert_eq!(back.rollovers.len(), 1);
        let zone = &back.rollovers[0];
        assert_eq!(zone.region, Rect { x: 4, y: 5, width: 2, height: 3 });
        assert_eq!(zone.pieces.len(), 2);

        // 2001: plane present, no encoded image.
        let p0 = &zone.pieces[0];
        assert_eq!(p0.kind, 0x67);
        assert_eq!(p0.raw.as_ref().expect("plane").bytes(), &[0b1011_0100]);
        assert!(p0.encoded.is_none());

        // 2002: plane + encoded image both present.
        let p1 = &zone.pieces[1];
        assert_eq!(p1.kind, 0x66);
        assert_eq!(p1.raw.as_ref().expect("plane").bytes(), &[0b0010_1101]);
        assert_eq!(
            p1.encoded.as_ref().expect("encoded").bytes(),
            &[0x89, b'P', b'N', b'G', 1, 2, 3, 4]
        );
    }

    /// Assert `back` round-tripped from [`sample_buttons`].
    pub(crate) fn assert_buttons(back: &[Button]) {
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].element_type, ElementType::Entry);
        assert_eq!(back[0].label.as_ref().unwrap().as_str().unwrap(), "réseau");
        assert!(back[0].concealed);
        assert_eq!(back[0].entry_state, 7);
        assert!(back[0].icon.is_none());

        assert_eq!(back[1].element_type, ElementType::FrogansSite);
        assert!(!back[1].concealed);
        assert_eq!(
            back[1].icon.as_ref().expect("icon").bytes(),
            &[1, 1, 2, 3, 5, 8]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{assert_representation, sample_representation};
    use super::*;

    #[test]
    fn representation_roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let rep = sample_representation(&pool);
        // SAFETY: `to_raw` allocated every image/array into `pool`.
        let back = unsafe { Representation::from_raw(rep.to_raw(&pool), &pool.as_pool()) };
        assert_representation(&back);
    }

    #[test]
    fn empty_rollovers_and_pieces_encode_null() {
        let pool = OwnedPool::new();
        let rep = Representation::new(&pool, 0, None, [0; 6], vec![]);
        let raw = rep.to_raw(&pool);
        assert!(raw.rollovers.is_null());
        assert_eq!(raw.rollover_count, 0);
        assert!(raw.image.buffer.is_null());

        let (count, ptr) = Button::list_to_raw(&[], &pool);
        assert_eq!(count, 0);
        assert!(ptr.is_null());
    }

    #[test]
    fn element_type_roundtrips() {
        for et in [
            ElementType::Base,
            ElementType::Entry,
            ElementType::FrogansSite,
            ElementType::ClipboardImage,
            ElementType::Other(0x12345),
        ] {
            assert_eq!(ElementType::from_raw(et.to_raw()), et);
        }
    }
}
