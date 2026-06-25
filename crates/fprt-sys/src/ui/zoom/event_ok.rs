//! `event_ok` payload (`0x0c`, IN) — the chosen zoom.

use crate::ui::EventTag;

/// The zoom the user committed: a type flag (default vs custom) and the value
/// in percent. `0x0c` bytes — no padding (three contiguous `i32`/`u32`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EventOk {
    /// Field 0 — host-written event tag (`EVT_OK`).
    pub event_id: EventTag,
    /// `0x3e9` = use default scaling factor, `0x3ea` = use `zoom_value`.
    pub zoom_type: i32,
    /// Scaling factor in percent, clamped 50..200 (only read when custom).
    pub zoom_value: i32,
}
