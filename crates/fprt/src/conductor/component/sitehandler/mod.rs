//! `sitehandler` — the Frogans Site viewport. Like [`inspector`](super::inspector)
//! it is **multi-instance**: every payload carries a [`SiteId`] (the site's
//! `data_subset` id) naming which native window it targets, so the lifecycle
//! commands thread the id rather than being bare markers.

use fprt_sys::ui::sitehandler::site_lifecycle::SiteLifecycle;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

mod cmd_update_layout;
mod cmd_update_visual;
mod evt_button_triggered;
mod evt_force_close;

pub use cmd_update_layout::UpdateLayout;
pub use cmd_update_visual::UpdateVisual;
pub use evt_button_triggered::ReportButtonTriggered;
pub use evt_force_close::ReportForceClose;

pub use fprt_core::component::sitehandler::SiteId;

/// A lifecycle/animation command (open/show/push/hide/close/begin/end) — the
/// whole payload is the [`SiteLifecycle`], so we surface just the id.
macro_rules! site_marker {
    ($name:ident, $id:expr, $export:ident, $variant:ident) => {
        /// A lifecycle command targeting one Frogans Site window.
        pub struct $name(pub SiteId);

        impl CommandPayload for $name {
            const ID: StatusName = $id;
            type Raw = SiteLifecycle;

            fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
                methods.$export
            }

            fn decode(raw: SiteLifecycle, _pool: &Pool) -> Command {
                Command::$variant(SiteId(raw.site_id))
            }
        }
    };
}

site_marker!(Open, fprt_sys::ui::sitehandler::CMD_OPEN, sitehandler_open, SitehandlerOpen);
site_marker!(Show, fprt_sys::ui::sitehandler::CMD_SHOW, sitehandler_show, SitehandlerShow);
site_marker!(Push, fprt_sys::ui::sitehandler::CMD_PUSH, sitehandler_push, SitehandlerPush);
site_marker!(Hide, fprt_sys::ui::sitehandler::CMD_HIDE, sitehandler_hide, SitehandlerHide);
site_marker!(Close, fprt_sys::ui::sitehandler::CMD_CLOSE, sitehandler_close, SitehandlerClose);
site_marker!(
    BeginAnimationInprogress,
    fprt_sys::ui::sitehandler::CMD_BEGIN_ANIMATION_INPROGRESS,
    sitehandler_begin_animation_inprogress,
    SitehandlerBeginAnimationInprogress
);
site_marker!(
    EndAnimationInprogress,
    fprt_sys::ui::sitehandler::CMD_END_ANIMATION_INPROGRESS,
    sitehandler_end_animation_inprogress,
    SitehandlerEndAnimationInprogress
);
