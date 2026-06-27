//! `update_visual` command (engine → host) — the rendered menu + entry buttons.
//!
//! COMPLEX (embeds visual `Representation` + `Button`): both directions.

use fprt_sys::ui::menu::CMD_UPDATE_VISUAL;
use fprt_sys::ui::menu::update_visual::UpdateVisual as Raw;

use crate::component::visual::{Button, Representation};
use crate::pool::{OwnedPool, Pool};

/// Which menu this visual is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuVariant {
    /// Global / pad menu (`0xfa1`).
    Global,
    /// Site menu (`0xfa2`) — see [`UpdateVisual::site_id`].
    Site,
    /// No menu (`800`).
    None,
    /// An engine value outside the documented set.
    Other(u32),
}

impl MenuVariant {
    /// Map the raw variant word.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0xfa1 => MenuVariant::Global,
            0xfa2 => MenuVariant::Site,
            800 => MenuVariant::None,
            other => MenuVariant::Other(other),
        }
    }

    /// Map back to the raw variant word.
    pub fn to_raw(self) -> u32 {
        match self {
            MenuVariant::Global => 0xfa1,
            MenuVariant::Site => 0xfa2,
            MenuVariant::None => 800,
            MenuVariant::Other(v) => v,
        }
    }
}

/// The menu's rendered representation plus its interactive entries.
#[derive(Debug)]
pub struct UpdateVisual {
    /// Which menu this is.
    pub variant: MenuVariant,
    /// Site id (meaningful only when `variant` is [`MenuVariant::Site`], else 0).
    pub site_id: u32,
    /// The rendered menu (background + rollover regions).
    pub representation: Representation,
    /// The interactive menu entries.
    pub buttons: Vec<Button>,
}

impl UpdateVisual {
    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: the representation + button array point into `pool`, the pool
        // that produced this pop.
        unsafe {
            UpdateVisual {
                variant: MenuVariant::from_raw(raw.variant),
                site_id: raw.site_id,
                representation: Representation::from_raw(raw.representation, pool),
                buttons: Button::list(raw.xbuttons, raw.xbutton_count.max(0) as usize, pool),
            }
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateVisual {
            variant: self.variant,
            site_id: self.site_id,
            representation: self.representation.copy_into(pool),
            buttons: self.buttons.iter().map(|b| b.copy_into(pool)).collect(),
        }
    }

    /// Encode into the raw payload, allocating the representation + button array
    /// into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        let (xbutton_count, xbuttons) = Button::list_to_raw(&self.buttons, pool);
        Raw {
            status_id: CMD_UPDATE_VISUAL,
            variant: self.variant.to_raw(),
            site_id: self.site_id,
            representation: self.representation.to_raw(pool),
            xbutton_count: xbutton_count as i32,
            xbuttons,
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
            variant: MenuVariant::Site,
            site_id: 42,
            representation: sample_representation(&pool),
            buttons: sample_buttons(&pool),
        };
        let raw = cmd.to_raw(&pool);
        assert_eq!(raw.status_id, CMD_UPDATE_VISUAL);
        let back = UpdateVisual::from_raw(raw, &pool.as_pool());
        assert_eq!(back.variant, MenuVariant::Site);
        assert_eq!(back.site_id, 42);
        assert_representation(&back.representation);
        assert_buttons(&back.buttons);
    }

    #[test]
    fn variant_roundtrips() {
        for v in [
            MenuVariant::Global,
            MenuVariant::Site,
            MenuVariant::None,
            MenuVariant::Other(0x999),
        ] {
            assert_eq!(MenuVariant::from_raw(v.to_raw()), v);
        }
    }
}
