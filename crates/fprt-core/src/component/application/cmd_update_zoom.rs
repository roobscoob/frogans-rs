//! `update_zoom` command (engine → host) — scalar, no pool.

use fprt_sys::ui::application::CMD_UPDATE_ZOOM;
use fprt_sys::ui::application::update_zoom::UpdateZoom as Raw;

/// The current zoom level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateZoom {
    /// Zoom level in percent (`100` = 100%).
    pub percent: i32,
}

impl UpdateZoom {
    /// Build one (no pool — scalar payload).
    pub fn new(percent: i32) -> Self {
        UpdateZoom { percent }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateZoom {
            percent: raw.zoom_level_percent,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_ZOOM,
            zoom_level_percent: self.percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let z = UpdateZoom::new(150);
        assert_eq!(UpdateZoom::from_raw(z.to_raw()), z);
    }
}
