//! `event_change_autosync` payload (`0x0c`, IN).

use crate::ui::EventTag;

/// The user toggled the inspector's autosync mode.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ChangeAutosync {
    pub event_id: EventTag,
    pub inspector_ref: i32,
    /// `0xbb9` = ON (mode 1), `0xbba` = OFF (mode 2).
    pub autosync_mode_enum: u32,
}
