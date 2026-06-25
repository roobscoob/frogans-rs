//! pad — the floating Frogans pad / launcher window (`fprt_ui_pad_*`).
//!
//! **Commands only — no inbound events.** 7 commands: 6 bare signals
//! (open/show/hide/close + begin/end animation, `Pop<StatusName>`) and
//! `update_layout` (the shared layout tuple). Command statuses `0x17f9xxxx`.

pub mod update_layout;

use crate::ui::{Pop, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x2195b5);
pub const CMD_UPDATE_LAYOUT: StatusName = StatusName(0x2195b6);
pub const CMD_BEGIN_ANIMATION: StatusName = StatusName(0x2195b7);
pub const CMD_END_ANIMATION: StatusName = StatusName(0x2195b8);
pub const CMD_SHOW: StatusName = StatusName(0x2195b9);
pub const CMD_HIDE: StatusName = StatusName(0x2195ba);
pub const CMD_CLOSE: StatusName = StatusName(0x2195bb);

// --- the 7 calls (all commands) ---
pub type OpenPop = Pop<StatusName>;
pub type ShowPop = Pop<StatusName>;
pub type HidePop = Pop<StatusName>;
pub type ClosePop = Pop<StatusName>;
pub type BeginAnimationPop = Pop<StatusName>;
pub type EndAnimationPop = Pop<StatusName>;
pub type UpdateLayoutPop = Pop<update_layout::UpdateLayout>;
