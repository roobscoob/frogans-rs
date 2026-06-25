//! `event_step_selected` payload (`0x0c`, IN).

use crate::ui::EventTag;

/// The user selected a step in the run-step list.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StepSelected {
    pub event_id: EventTag,
    pub inspector_ref: i32,
    pub step_index: i32,
}
