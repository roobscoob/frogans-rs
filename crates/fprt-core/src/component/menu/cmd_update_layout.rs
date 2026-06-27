//! `update_layout` command (engine → host) — menu geometry (host-discarded).

use fprt_sys::ui::layout_tuple::LayoutTuple;
use fprt_sys::ui::menu::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::menu::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::sld_rect::SldRect;

use crate::component::visual::ScreenRect;

/// The menu's geometry (`None` ⇒ no rect supplied). The macOS host discards this
/// — the menu self-positions at the cursor — but it is modeled for completeness.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateLayout {
    /// Where the engine would place the menu.
    pub rect: Option<ScreenRect>,
}

impl UpdateLayout {
    /// Build one (no pool — geometry payload).
    pub fn new(rect: Option<ScreenRect>) -> Self {
        UpdateLayout { rect }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateLayout {
            rect: ScreenRect::option(raw.menu_layout.present_flag, raw.menu_layout.rect),
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        let menu_layout = match self.rect {
            Some(r) => LayoutTuple {
                present_flag: 1,
                rect: SldRect {
                    screen_index: r.screen_index,
                    reserved: 0,
                    x: r.x,
                    y: r.y,
                },
            },
            None => LayoutTuple {
                present_flag: 0,
                rect: SldRect {
                    screen_index: 0,
                    reserved: 0,
                    x: 0,
                    y: 0,
                },
            },
        };
        Raw {
            status_id: CMD_UPDATE_LAYOUT,
            menu_layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_present_and_absent() {
        for rect in [
            Some(ScreenRect {
                screen_index: 1,
                x: 40,
                y: 50,
            }),
            None,
        ] {
            let l = UpdateLayout::new(rect);
            assert_eq!(UpdateLayout::from_raw(l.to_raw()), l);
        }
    }
}
