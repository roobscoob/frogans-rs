//! `update_sync` command (engine → host) — autosync state + synchronize-enable.

use fprt_sys::ui::inspector::CMD_UPDATE_SYNC;
use fprt_sys::ui::inspector::update_sync::UpdateSync as Raw;

use crate::component::inspector::InspectorId;

/// The inspector's auto-sync state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateSync {
    /// The target window.
    pub id: InspectorId,
    /// Autosync button polarity: `true` = ON (`0xbb9`), `false` = OFF (`0xbba`).
    pub autosync_on: bool,
    /// Whether the Synchronize button is enabled.
    pub synchronize_enabled: bool,
}

impl UpdateSync {
    /// Build one (no pool — scalar payload).
    pub fn new(id: InspectorId, autosync_on: bool, synchronize_enabled: bool) -> Self {
        UpdateSync {
            id,
            autosync_on,
            synchronize_enabled,
        }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateSync {
            id: InspectorId(raw.reference),
            autosync_on: raw.autosync_mode == 0xbb9,
            synchronize_enabled: raw.synchronize_enabled != 0,
        }
    }

    /// Encode into the raw payload (`0xbb9` = ON, `0xbba` = OFF).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_SYNC,
            reference: self.id.0,
            autosync_mode: if self.autosync_on { 0xbb9 } else { 0xbba },
            synchronize_enabled: self.synchronize_enabled as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_every_combination() {
        for on in [true, false] {
            for enabled in [true, false] {
                let p = UpdateSync::new(InspectorId(6), on, enabled);
                assert_eq!(UpdateSync::from_raw(p.to_raw()), p);
            }
        }
    }
}
