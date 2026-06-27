//! `change_layout` event (host → engine) — **dormant**, and the one *owned*
//! event (it carries a `Vec`), so its `to_raw` takes a scratch [`OwnedPool`] to
//! hold the converted sitehandler array for the call.
//!
//! Symmetric codec: `to_raw` encodes into a scratch [`OwnedPool`]; `from_raw`
//! reads the raw's owned sitehandler array back into a `Vec`.

use fprt_sys::ui::application::EVT_CHANGE_LAYOUT;
use fprt_sys::ui::application::event_change_layout::{ChangeLayoutSitehandler, EventChangeLayout};
use fprt_sys::ui::layout_tuple::LayoutTuple;
use fprt_sys::ui::sld_rect::SldRect;

use crate::component::visual::ScreenRect;
use crate::pool::OwnedPool;

/// A pad/menu layout sub-object: whether it changed and, if so, its rect (`None`
/// ⇒ changed but no rect supplied).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LayoutChange {
    /// This sub-object did not change.
    #[default]
    Unchanged,
    /// This sub-object changed to the given rect (`None` ⇒ no rect).
    Changed(Option<ScreenRect>),
}

/// One sitehandler's layout change.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SitehandlerLayout {
    /// data-subset id / reference.
    pub id: u32,
    /// New rect (`None` ⇒ no rect supplied).
    pub rect: Option<ScreenRect>,
    /// Zoom / user-size scale level (not pixels).
    pub user_size: i32,
}

/// Report an application-wide layout change. Each field is independently gated;
/// `ReportChangeLayout::default()` is an all-unchanged no-op.
#[derive(Default, PartialEq, Debug)]
pub struct ReportChangeLayout {
    /// New application layout scalar, if it changed (meaning unresolved).
    pub layout: Option<u32>,
    /// Per-sitehandler layout changes (empty ⇒ none).
    pub sitehandlers: Vec<SitehandlerLayout>,
    /// The pad's layout change.
    pub pad: LayoutChange,
    /// The menu's layout change.
    pub menu: LayoutChange,
}

/// `present_flag` + `SldRect` from an optional rect.
fn sld(rect: Option<ScreenRect>) -> (u32, SldRect) {
    match rect {
        Some(r) => (
            1,
            SldRect {
                screen_index: r.screen_index,
                reserved: 0,
                x: r.x,
                y: r.y,
            },
        ),
        None => (
            0,
            SldRect {
                screen_index: 0,
                reserved: 0,
                x: 0,
                y: 0,
            },
        ),
    }
}

/// `change_occured` + `LayoutTuple` from a [`LayoutChange`].
fn layout_tuple(change: LayoutChange) -> (u32, LayoutTuple) {
    match change {
        LayoutChange::Unchanged => {
            let (present_flag, rect) = sld(None);
            (0, LayoutTuple { present_flag, rect })
        }
        LayoutChange::Changed(rect) => {
            let (present_flag, rect) = sld(rect);
            (1, LayoutTuple { present_flag, rect })
        }
    }
}

/// An optional rect from a `present_flag` + `SldRect` (inverse of [`sld`]).
fn screen_rect(present_flag: u32, rect: SldRect) -> Option<ScreenRect> {
    (present_flag != 0).then_some(ScreenRect {
        screen_index: rect.screen_index,
        x: rect.x,
        y: rect.y,
    })
}

/// A [`LayoutChange`] from a `change_occured` + `LayoutTuple` (inverse of
/// [`layout_tuple`]).
fn layout_change(change_occured: u32, tuple: LayoutTuple) -> LayoutChange {
    if change_occured == 0 {
        LayoutChange::Unchanged
    } else {
        LayoutChange::Changed(screen_rect(tuple.present_flag, tuple.rect))
    }
}

impl ReportChangeLayout {
    /// An all-unchanged event (equivalent to [`Default`]).
    pub fn new() -> Self {
        ReportChangeLayout::default()
    }

    /// Encode into the raw payload, allocating the sitehandler array into
    /// `scratch` (which must outlive the call the raw is passed to).
    pub fn to_raw(&self, scratch: &OwnedPool) -> EventChangeLayout {
        let sites: Vec<ChangeLayoutSitehandler> = self
            .sitehandlers
            .iter()
            .map(|s| {
                let (present_flag, rect) = sld(s.rect);
                ChangeLayoutSitehandler {
                    id: s.id,
                    present_flag,
                    rect,
                    user_size: s.user_size,
                }
            })
            .collect();
        let sitehandlers = if sites.is_empty() {
            core::ptr::null_mut()
        } else {
            scratch.alloc_slice(&sites).cast::<ChangeLayoutSitehandler>() as *mut _
        };

        let (layout_present, layout_scalar) = match self.layout {
            Some(v) => (1, v),
            None => (0, 0),
        };
        let (pad_change_occured, pad_layout) = layout_tuple(self.pad);
        let (menu_change_occured, menu_layout) = layout_tuple(self.menu);

        EventChangeLayout {
            event_id: EVT_CHANGE_LAYOUT,
            layout_present,
            layout_scalar,
            sitehandlers_present: u32::from(!sites.is_empty()),
            sitehandlers_count: sites.len() as i32,
            sitehandlers,
            pad_change_occured,
            pad_layout,
            menu_change_occured,
            menu_layout,
        }
    }

    /// Decode from the raw payload, reading the sitehandler pointer-array into
    /// an owned `Vec`.
    pub fn from_raw(raw: &EventChangeLayout) -> Self {
        let layout = if raw.layout_present != 0 {
            Some(raw.layout_scalar)
        } else {
            None
        };

        let mut sitehandlers = Vec::new();
        if raw.sitehandlers_present != 0 && !raw.sitehandlers.is_null() {
            let count = raw.sitehandlers_count.max(0) as usize;
            sitehandlers.reserve(count);
            for i in 0..count {
                // SAFETY: `sitehandlers_present` is set and the pointer is
                // non-null, so `sitehandlers` points at `sitehandlers_count`
                // valid records (the array `to_raw` allocated into the pool).
                let s = unsafe { *raw.sitehandlers.add(i) };
                sitehandlers.push(SitehandlerLayout {
                    id: s.id,
                    rect: screen_rect(s.present_flag, s.rect),
                    user_size: s.user_size,
                });
            }
        }

        ReportChangeLayout {
            layout,
            sitehandlers,
            pad: layout_change(raw.pad_change_occured, raw.pad_layout),
            menu: layout_change(raw.menu_change_occured, raw.menu_layout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let x = ReportChangeLayout {
            layout: Some(0x2a),
            sitehandlers: vec![
                SitehandlerLayout {
                    id: 7,
                    rect: Some(ScreenRect {
                        screen_index: 1,
                        x: 10,
                        y: 20,
                    }),
                    user_size: 3,
                },
                SitehandlerLayout {
                    id: 8,
                    rect: None,
                    user_size: -1,
                },
            ],
            pad: LayoutChange::Changed(Some(ScreenRect {
                screen_index: 0,
                x: 5,
                y: 6,
            })),
            menu: LayoutChange::Unchanged,
        };

        let scratch = OwnedPool::new();
        let raw = x.to_raw(&scratch);
        assert_eq!(ReportChangeLayout::from_raw(&raw), x);
    }

    #[test]
    fn default_is_all_unchanged() {
        let x = ReportChangeLayout::default();
        let scratch = OwnedPool::new();
        let raw = x.to_raw(&scratch);
        assert_eq!(ReportChangeLayout::from_raw(&raw), x);
    }
}
