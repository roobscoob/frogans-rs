//! `sitehandler` — the Frogans Site viewport. Multi-instance: every payload
//! carries a [`SiteId`] naming which native window it targets.
//!
//! `update_visual` is COMPLEX (embeds visual `Representation` + `Button`):
//! decode-only for now. `update_layout` and the two events are full codecs. The
//! lifecycle commands are bare `SiteId`-carrying markers handled by transport.

mod cmd_update_layout;
mod cmd_update_visual;
mod evt_button_triggered;
mod evt_force_close;

pub use cmd_update_layout::UpdateLayout;
pub use cmd_update_visual::UpdateVisual;
pub use evt_button_triggered::ReportButtonTriggered;
pub use evt_force_close::ReportForceClose;

/// Which Frogans Site a command targets / an event came from (the engine
/// `data_subset` id). `Copy + Eq + Hash` so consumers can key a window map by it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SiteId(pub i32);
