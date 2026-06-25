//! `event_content_selected` payload (`0x0c`, IN).

use crate::ui::EventTag;

/// The user selected a content entry.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ContentSelected {
    pub event_id: EventTag,
    pub inspector_ref: i32,
    pub content_index: i32,
}
