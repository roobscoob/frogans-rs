//! `update_layout` command (engine → host) — re-position / zoom a site window.

use fprt_sys::ui::sitehandler::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::sitehandler::update_layout::UpdateLayout as Raw;
use fprt_sys::ui::sld_rect::SldRect;

use crate::component::sitehandler::SiteId;
use crate::component::visual::ScreenRect;

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

impl UpdateLayout {
    /// Build one (no pool — geometry payload).
    pub fn new(id: SiteId, rect: Option<ScreenRect>, user_size: i32) -> Self {
        UpdateLayout {
            id,
            rect,
            user_size,
        }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateLayout {
            id: SiteId(raw.site_id),
            rect: ScreenRect::option(raw.present_flag, raw.rect),
            user_size: raw.user_size,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        let (present_flag, rect) = match self.rect {
            Some(r) => (
                1,
                SldRect {
                    screen_index: r.screen_index,
                    reserved: 0,
                    x: r.x,
                    y: r.y,
                },
            ),
            None => (
                0,
                SldRect {
                    screen_index: 0,
                    reserved: 0,
                    x: 0,
                    y: 0,
                },
            ),
        };
        Raw {
            status_id: CMD_UPDATE_LAYOUT,
            site_id: self.id.0,
            present_flag,
            rect,
            user_size: self.user_size,
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
                screen_index: 0,
                x: 12,
                y: 34,
            }),
            None,
        ] {
            let l = UpdateLayout::new(SiteId(9), rect, 150);
            assert_eq!(UpdateLayout::from_raw(l.to_raw()), l);
        }
    }
}
