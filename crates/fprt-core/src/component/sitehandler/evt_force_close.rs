//! `force_close` event (host → engine) — force a site closed. Dormant on macOS.

use fprt_sys::ui::sitehandler::EVT_FORCE_CLOSE;
use fprt_sys::ui::sitehandler::force_close::ForceClose as Raw;

use crate::component::sitehandler::SiteId;

/// Force a Frogans Site closed. Carries only the site id. (Dormant on the macOS
/// host — no sender is wired there — but modeled for completeness.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportForceClose {
    /// The site to force-close.
    pub id: SiteId,
}

impl ReportForceClose {
    /// Force-close site `id` (no pool — scalar payload).
    pub fn new(id: SiteId) -> Self {
        ReportForceClose { id }
    }

    /// Decode an inbound payload.
    pub fn from_raw(raw: &Raw) -> Self {
        ReportForceClose {
            id: SiteId(raw.site_id),
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_FORCE_CLOSE,
            site_id: self.id.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let evt = ReportForceClose::new(SiteId(7));
        assert_eq!(ReportForceClose::from_raw(&evt.to_raw()), evt);
    }
}
