//! `command_update_content_labels` payload (`0x20`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The inspector content selector's entries + the active index.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateContentLabels {
    pub status_id: StatusName,
    pub reference: i32,
    /// Number of content-label entries.
    pub count: u32,
    pub _rsv0c: u32,
    /// Mempool array of `count` labels (stride `0x10`).
    pub labels: *const Ustring,
    /// Selected/active content index.
    pub content_active: i32,
    pub _rsv1c: u32,
}
