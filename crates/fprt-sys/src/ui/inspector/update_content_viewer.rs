//! `command_update_content_viewer` payload (`0x28`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// One content document loaded into the inspector's viewer, with a syntax mode.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateContentViewer {
    pub status_id: StatusName,
    pub reference: i32,
    /// `0x7d1` json / `0x7d2` xml / `0x7d3` uxce / `0x7d0` default (bin/unknown).
    pub content_mode: u32,
    pub _rsv0c: u32,
    /// Document text.
    pub content: Ustring,
    /// Trailing selector/position int (engine emits 0 / -1).
    pub content_select: i32,
    pub _rsv24: u32,
}
