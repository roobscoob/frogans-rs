//! `event_menu_access_wanted` payload (`0x0c`, IN).

use crate::ui::EventTag;
use crate::ui::application::menu_variant::MenuVariant;

/// The UI wants the application menu shown (globally or for a specific site).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MenuAccessWanted {
    /// Field 0 — must be `EVT_MENU_ACCESS_WANTED` (`0x10cccd`).
    pub event_id: EventTag,
    pub variant: MenuVariant,
    /// Site id when `variant == MenuVariant::SITE`, else `0` / `-1`.
    pub site_id: u32,
}
