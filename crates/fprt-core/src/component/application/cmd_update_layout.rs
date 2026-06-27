//! `update_layout` command (engine → host) — scalar passthrough, no pool.

use fprt_sys::ui::application::CMD_UPDATE_LAYOUT;
use fprt_sys::ui::application::update_layout::UpdateLayout as Raw;

/// An application-level layout value (meaning unresolved — passthrough).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateLayout {
    /// Application layout value.
    pub layout_scalar: u32,
}

impl UpdateLayout {
    /// Build one (no pool — scalar payload).
    pub fn new(layout_scalar: u32) -> Self {
        UpdateLayout { layout_scalar }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateLayout {
            layout_scalar: raw.layout_scalar,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LAYOUT,
            layout_scalar: self.layout_scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let l = UpdateLayout::new(42);
        assert_eq!(UpdateLayout::from_raw(l.to_raw()), l);
    }
}
