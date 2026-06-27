//! `update_visual` command (engine → host) — the rendered site slides + zones.
//!
//! COMPLEX (embeds visual `Representation` + `Button`): both directions.

use fprt_sys::ui::sitehandler::CMD_UPDATE_VISUAL;
use fprt_sys::ui::sitehandler::update_visual::UpdateVisual as Raw;

use crate::component::sitehandler::SiteId;
use crate::component::visual::{Button, Representation};
use crate::pool::{OwnedPool, Pool};

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

impl UpdateVisual {
    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
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

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateVisual {
            id: self.id,
            vignette: self.vignette.copy_into(pool),
            lead: self.lead.copy_into(pool),
            buttons: self.buttons.iter().map(|b| b.copy_into(pool)).collect(),
        }
    }

    /// Encode into the raw payload, allocating both slides + the button array
    /// into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        let (button_count, buttons) = Button::list_to_raw(&self.buttons, pool);
        Raw {
            status_id: CMD_UPDATE_VISUAL,
            site_id: self.id.0,
            vignette: self.vignette.to_raw(pool),
            lead: self.lead.to_raw(pool),
            button_count: button_count as u32,
            buttons,
            _rsv_a8: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::visual::test_support::{
        assert_buttons, assert_representation, sample_buttons, sample_representation,
    };

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateVisual {
            id: SiteId(9),
            vignette: sample_representation(&pool),
            lead: sample_representation(&pool),
            buttons: sample_buttons(&pool),
        };
        let raw = cmd.to_raw(&pool);
        assert_eq!(raw.status_id, CMD_UPDATE_VISUAL);
        let back = UpdateVisual::from_raw(raw, &pool.as_pool());
        assert_eq!(back.id, SiteId(9));
        assert_representation(&back.vignette);
        assert_representation(&back.lead);
        assert_buttons(&back.buttons);
    }
}
