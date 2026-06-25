//! The bare `{ event_id, inspector_ref }` event (`8` bytes, IN).

use crate::ui::EventTag;

/// The payload for `synchronize`, `rerun`, and `close` — the event tag plus the
/// target inspector instance reference (no value field).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RefEvent {
    /// Field 0 — host-written event tag.
    pub event_id: EventTag,
    /// Inspector instance reference the event came from.
    pub inspector_ref: i32,
}
