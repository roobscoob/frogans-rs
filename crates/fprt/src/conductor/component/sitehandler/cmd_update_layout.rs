//! `update_layout` command (engine → host) — re-position / zoom a site window.

use fprt_sys::ui::sitehandler::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::sitehandler::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::sitehandler::SiteId;
use crate::conductor::component::visual::ScreenRect;
use crate::pool::Pool;

/// Re-position / zoom one Frogans Site's native window.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateLayout {
    /// The target site window.
    pub id: SiteId,
    /// Where to place it (`None` ⇒ host centers the site & ignores position).
    pub rect: Option<ScreenRect>,
    /// Zoom / user-size scale level (not pixels).
    pub user_size: i32,
}

impl CommandPayload for UpdateLayout {
    const ID: StatusName = CMD_UPDATE_LAYOUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.sitehandler_update_layout
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateLayout {
            id: SiteId(raw.site_id),
            rect: ScreenRect::option(raw.present_flag, raw.rect),
            user_size: raw.user_size,
        }
    }

    fn into_command(self) -> Command {
        Command::SitehandlerUpdateLayout(self)
    }
}
