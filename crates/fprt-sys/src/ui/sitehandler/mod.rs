//! sitehandler — the Frogans Site viewport (`fprt_ui_sitehandler_*`).
//!
//! 9 commands (engine → host, [`Pop`]) + 2 events (host → engine, [`Report`]).
//! Each site renders in its own native window; `update_visual` carries the
//! rendered slides + zone map, `update_layout` the window rect. Command statuses
//! `0x1882xxxx`, event statuses `0x1880xxxx`.

pub mod button_triggered;
pub mod force_close;
pub mod site_lifecycle;
pub mod update_layout;
pub mod update_visual;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags (engine stamps payload field 0) ---
pub const CMD_OPEN: StatusName = StatusName(0x2195fa);
pub const CMD_UPDATE_LAYOUT: StatusName = StatusName(0x2195fb);
pub const CMD_UPDATE_VISUAL: StatusName = StatusName(0x2195fc);
pub const CMD_BEGIN_ANIMATION_INPROGRESS: StatusName = StatusName(0x2195fd);
pub const CMD_END_ANIMATION_INPROGRESS: StatusName = StatusName(0x2195fe);
pub const CMD_SHOW: StatusName = StatusName(0x2195ff);
pub const CMD_PUSH: StatusName = StatusName(0x219600);
pub const CMD_HIDE: StatusName = StatusName(0x219601);
pub const CMD_CLOSE: StatusName = StatusName(0x219602);

// --- event tags (host writes payload field 0) ---
pub const EVT_BUTTON_TRIGGERED: EventTag = EventTag(0x10cced);
pub const EVT_FORCE_CLOSE: EventTag = EventTag(0x10ccee);

// --- the 11 calls ---
// commands (engine → host); the 7 lifecycle/animation calls share `SiteLifecycle`
pub type OpenPop = Pop<site_lifecycle::SiteLifecycle>;
pub type ClosePop = Pop<site_lifecycle::SiteLifecycle>;
pub type ShowPop = Pop<site_lifecycle::SiteLifecycle>;
pub type HidePop = Pop<site_lifecycle::SiteLifecycle>;
pub type BeginAnimationInprogressPop = Pop<site_lifecycle::SiteLifecycle>;
pub type EndAnimationInprogressPop = Pop<site_lifecycle::SiteLifecycle>;
pub type PushPop = Pop<site_lifecycle::SiteLifecycle>;
pub type UpdateLayoutPop = Pop<update_layout::UpdateLayout>;
pub type UpdateVisualPop = Pop<update_visual::UpdateVisual>;
// events (host → engine)
pub type ButtonTriggeredReport = Report<button_triggered::ButtonTriggered>;
pub type ForceCloseReport = Report<force_close::ForceClose>;
