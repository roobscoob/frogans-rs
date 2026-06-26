//! `update_visual` command (engine → host) — the rendered site slides + zones.

use fprt_sys::ui::sitehandler::update_visual::UpdateVisual as Raw;
use fprt_sys::ui::sitehandler::CMD_UPDATE_VISUAL;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::sitehandler::SiteId;
use crate::conductor::component::visual::{Button, Representation};
use crate::pool::Pool;

/// The site's rendered visual scheme: the vignette + lead slides and the
/// per-element interactive zone list.
#[derive(Debug)]
pub struct UpdateVisual {
    /// The target site window.
    pub id: SiteId,
    /// Vignette slide.
    pub vignette: Representation,
    /// Lead slide.
    pub lead: Representation,
    /// The interactive zone elements.
    pub buttons: Vec<Button>,
}

impl CommandPayload for UpdateVisual {
    const ID: StatusName = CMD_UPDATE_VISUAL;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.sitehandler_update_visual
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: the slides + button array point into `pool`, the pool that
        // produced this pop.
        unsafe {
            UpdateVisual {
                id: SiteId(raw.site_id),
                vignette: Representation::from_raw(raw.vignette, pool),
                lead: Representation::from_raw(raw.lead, pool),
                buttons: Button::list(raw.buttons, raw.button_count as usize, pool),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::SitehandlerUpdateVisual(self)
    }
}
