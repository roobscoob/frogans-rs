//! `event_change_layout` payload (`0x50`, IN) — **dormant**.
//!
//! A fully-implemented engine entry the shipped macOS host never sends (no
//! sender is wired; whether any Windows host sends it is unresolved). Modelled
//! for contract completeness. Carries four independent layout sub-objects, each
//! gated by its own present/change flag.

use crate::ui::layout_tuple::LayoutTuple;
use crate::ui::sld_rect::SldRect;
use crate::ui::EventTag;

/// One sitehandler layout element in [`EventChangeLayout::sitehandlers`] (`0x1c`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ChangeLayoutSitehandler {
    /// data-subset id / reference.
    pub id: u32,
    pub present_flag: u32,
    pub rect: SldRect,
    /// Zoom / user-size scale level (not pixels).
    pub user_size: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EventChangeLayout {
    /// Field 0 — must be `EVT_CHANGE_LAYOUT` (`0x10ccd2`).
    pub event_id: EventTag,
    pub layout_present: u32,
    /// Application layout value (→ dwh+0x48). Meaning unresolved.
    pub layout_scalar: u32,
    pub sitehandlers_present: u32,
    pub sitehandlers_count: i32,
    // +0x14: 4 bytes implicit padding → sitehandlers aligns to +0x18.
    pub sitehandlers: *mut ChangeLayoutSitehandler,
    pub pad_change_occured: u32,
    pub pad_layout: LayoutTuple,
    pub menu_change_occured: u32,
    pub menu_layout: LayoutTuple,
}
