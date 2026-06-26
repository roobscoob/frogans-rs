//! `update_visual` command (engine → host) — the rendered menu + entry buttons.

use fprt_sys::ui::menu::update_visual::UpdateVisual as Raw;
use fprt_sys::ui::menu::CMD_UPDATE_VISUAL;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::visual::{Button, Representation};
use crate::pool::Pool;

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
    fn from_raw(raw: u32) -> Self {
        match raw {
            0xfa1 => MenuVariant::Global,
            0xfa2 => MenuVariant::Site,
            800 => MenuVariant::None,
            other => MenuVariant::Other(other),
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

impl CommandPayload for UpdateVisual {
    const ID: StatusName = CMD_UPDATE_VISUAL;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.menu_update_visual
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
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

    fn into_command(self) -> Command {
        Command::MenuUpdateVisual(self)
    }
}
