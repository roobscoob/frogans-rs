//! `menu_access_wanted` event (host → engine) — an enum, no string.

use fprt_sys::ui::application::EVT_MENU_ACCESS_WANTED;
use fprt_sys::ui::application::event_menu_access_wanted::MenuAccessWanted as Raw;
use fprt_sys::ui::application::menu_variant::MenuVariant;

/// Which application menu the UI wants shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuTarget {
    /// The global menu (no specific site).
    Global,
    /// The menu for a specific site (`site_id`).
    Site(u32),
}

/// The UI wants the application menu shown — globally or for a specific site.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReportMenuAccessWanted {
    /// Which menu to show.
    pub target: MenuTarget,
}

impl ReportMenuAccessWanted {
    /// Request the menu for `target`.
    pub fn new(target: MenuTarget) -> Self {
        ReportMenuAccessWanted { target }
    }

    /// Decode an inbound payload.
    pub fn from_raw(raw: &Raw) -> Self {
        let target = match raw.variant {
            MenuVariant::SITE => MenuTarget::Site(raw.site_id),
            _ => MenuTarget::Global,
        };
        ReportMenuAccessWanted { target }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        let (variant, site_id) = match self.target {
            MenuTarget::Global => (MenuVariant::GLOBAL, 0),
            MenuTarget::Site(id) => (MenuVariant::SITE, id),
        };
        Raw {
            event_id: EVT_MENU_ACCESS_WANTED,
            variant,
            site_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_both_targets() {
        for t in [MenuTarget::Global, MenuTarget::Site(7)] {
            let evt = ReportMenuAccessWanted::new(t);
            assert_eq!(ReportMenuAccessWanted::from_raw(&evt.to_raw()), evt);
        }
    }
}
