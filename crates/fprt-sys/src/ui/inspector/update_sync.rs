//! `command_update_sync` payload (`0x10`).

use crate::ui::StatusName;

/// The inspector auto-sync state: button polarity + synchronize-enable flag.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateSync {
    pub status_id: StatusName,
    pub reference: i32,
    /// `0xbb9` = ON (mode 1), `0xbba` = OFF (mode 2).
    pub autosync_mode: u32,
    /// Nonzero ⇒ enable the Synchronize button.
    pub synchronize_enabled: u32,
}
