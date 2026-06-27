//! `update_layout` command (engine → host) — the pad window's geometry.

use fprt_sys::ui::layout_tuple::LayoutTuple;
use fprt_sys::ui::pad::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::pad::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::sld_rect::SldRect;

use crate::component::visual::ScreenRect;

/// The pad window's on-screen rectangle (`None` ⇒ no rect supplied). On the
/// macOS build the host discards this; modeled for completeness.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateLayout {
    /// Where to place the pad window.
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
            rect: ScreenRect::option(raw.layout.present_flag, raw.layout.rect),
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        let layout = match self.rect {
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
            layout,
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
                screen_index: 2,
                x: 100,
                y: 200,
            }),
            None,
        ] {
            let l = UpdateLayout::new(rect);
            assert_eq!(UpdateLayout::from_raw(l.to_raw()), l);
        }
    }
}
