//! The `MenuVariant` selector (`event_menu_access_wanted`).

/// Which application menu the UI wants shown.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MenuVariant(pub u32);

impl MenuVariant {
    /// `0xfa1` — global menu (no specific site).
    pub const GLOBAL: MenuVariant = MenuVariant(0xfa1);
    /// `0xfa2` — site menu (`site_id` identifies the site).
    pub const SITE: MenuVariant = MenuVariant(0xfa2);
}
