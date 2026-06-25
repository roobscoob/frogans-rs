//! menu — application / context menus (`fprt_ui_menu_*`).
//!
//! 7 commands (engine → host, [`Pop`]) + 1 event (host → engine, [`Report`]).
//! The menu is engine-rendered SLD content (not a native window): the 5
//! lifecycle commands are bare 4-byte triggers, `update_visual` carries the
//! rendered pixels, and the host discards `update_layout`. Command statuses
//! `0x1808xxxx`, event statuses `0x1806d5xx`.

pub mod button_triggered;
pub mod update_layout;
pub mod update_visual;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags (engine stamps payload field 0) ---
pub const CMD_OPEN: StatusName = StatusName(0x2195bc);
pub const CMD_UPDATE_VISUAL: StatusName = StatusName(0x2195bd);
pub const CMD_UPDATE_LAYOUT: StatusName = StatusName(0x2195be);
pub const CMD_SHOW: StatusName = StatusName(0x2195bf);
pub const CMD_PUSH: StatusName = StatusName(0x2195c0);
pub const CMD_HIDE: StatusName = StatusName(0x2195c1);
pub const CMD_CLOSE: StatusName = StatusName(0x2195c2);

// --- event tag (host writes payload field 0) ---
pub const EVT_BUTTON_TRIGGERED: EventTag = EventTag(0x10ccd4);

// --- the 8 calls ---
// commands (engine → host); the 5 lifecycle commands are bare 4-byte triggers
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type PushPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type UpdateVisualPop = Pop<update_visual::UpdateVisual>;
pub type UpdateLayoutPop = Pop<update_layout::UpdateLayout>;
// event (host → engine)
pub type ButtonTriggeredReport = Report<button_triggered::ButtonTriggered>;
