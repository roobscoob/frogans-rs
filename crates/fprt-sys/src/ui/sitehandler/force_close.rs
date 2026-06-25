//! `event_force_close` payload (`0x08`, IN) — **dormant** on macOS.

use crate::ui::EventTag;

/// Force a Frogans Site closed. Carries only the site id — no button, no text,
/// no reason code. Dormant on the macOS host (no sender wired); a Windows sender
/// is unresolved.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ForceClose {
    /// Field 0 — must be `EVT_FORCE_CLOSE` (`0x10ccee`).
    pub event_id: EventTag,
    /// `data_subset` id of the FSI to force-close.
    pub site_id: i32,
}
