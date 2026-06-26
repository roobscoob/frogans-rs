//! `menu_access_wanted` event (host → engine) — the UI wants the menu shown.

use fprt_sys::ui::application::event_menu_access_wanted::MenuAccessWanted;
use fprt_sys::ui::application::menu_variant::MenuVariant;
use fprt_sys::ui::application::EVT_MENU_ACCESS_WANTED;

use crate::call::invoke;
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;

/// Which application menu the UI wants shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuTarget {
    /// The global menu (no specific site).
    Global,
    /// The menu for a specific site (`site_id`).
    Site(u32),
}

/// The UI wants the application menu shown — globally or for a specific site.
pub struct ReportMenuAccessWanted {
    target: MenuTarget,
}

impl ReportMenuAccessWanted {
    /// Request the menu for `target`.
    pub fn new(target: MenuTarget) -> Self {
        ReportMenuAccessWanted { target }
    }
}

impl sealed::Sealed for ReportMenuAccessWanted {}

impl Report for ReportMenuAccessWanted {
    fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
        let engine = conductor.engine();
        let ctx = conductor.ctx();
        let (variant, site_id) = match self.target {
            MenuTarget::Global => (MenuVariant::GLOBAL, 0),
            MenuTarget::Site(id) => (MenuVariant::SITE, id),
        };
        let payload = MenuAccessWanted {
            event_id: EVT_MENU_ACCESS_WANTED,
            variant,
            site_id,
        };
        // SAFETY: valid ctx; `payload` outlives the call.
        invoke(engine, |s, e, p| unsafe {
            (engine.methods().application_menu_access_wanted)(ctx, &payload, s, e, p)
        })
        .map(|_| ())
    }
}
