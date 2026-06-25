//! The shared `{ status_id, site_id }` payload (`0x08`) for the sitehandler
//! lifecycle + animation commands (open / close / show / hide / begin_anim /
//! end_anim / push).

use crate::ui::StatusName;

/// Lifecycle/animation command payload: the engine-stamped status name plus the
/// `site_id` the host routes the per-site native-window side effect to.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SiteLifecycle {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// `data_subset` id of the target Frogans Site.
    pub site_id: i32,
}
