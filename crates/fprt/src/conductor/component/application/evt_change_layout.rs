//! `change_layout` event (host → engine) — **dormant**.
//!
//! A fully-implemented engine entry the shipped macOS host never sends (no sender
//! is wired). Modeled for contract completeness: it carries four independent
//! layout sub-objects, each gated by its own present/change flag.

use fprt_sys::ui::application::event_change_layout::{ChangeLayoutSitehandler, EventChangeLayout};
use fprt_sys::ui::application::EVT_CHANGE_LAYOUT;
use fprt_sys::ui::layout_tuple::LayoutTuple;
use fprt_sys::ui::sld_rect::SldRect;

use crate::call::invoke;
use crate::conductor::component::visual::ScreenRect;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

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
#[derive(Default)]
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

impl ReportChangeLayout {
    /// An all-unchanged event (equivalent to [`Default`]).
    pub fn new() -> Self {
        ReportChangeLayout::default()
    }
}

impl sealed::Sealed for ReportChangeLayout {}

impl Report for ReportChangeLayout {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();

        // The sitehandler array must outlive the call (the engine copies during it).
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

        let (layout_present, layout_scalar) = match self.layout {
            Some(v) => (1, v),
            None => (0, 0),
        };
        let (pad_change_occured, pad_layout) = layout_tuple(self.pad);
        let (menu_change_occured, menu_layout) = layout_tuple(self.menu);

        let payload = EventChangeLayout {
            event_id: EVT_CHANGE_LAYOUT,
            layout_present,
            layout_scalar,
            sitehandlers_present: u32::from(!sites.is_empty()),
            sitehandlers_count: sites.len() as i32,
            sitehandlers: sites.as_ptr() as *mut ChangeLayoutSitehandler,
            pad_change_occured,
            pad_layout,
            menu_change_occured,
            menu_layout,
        };
        // SAFETY: valid ctx; `payload` + `sites` outlive the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().application_change_layout)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
