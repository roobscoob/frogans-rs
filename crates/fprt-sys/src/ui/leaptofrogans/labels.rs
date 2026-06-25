//! `command_update_labels` payload (`0x78`) — title + seven button strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The leap-to-Frogans dialog's localized strings. **Two-armed**: the engine
/// fills the normal-view buttons (confirm/cancel/block) OR the error-view button
/// (close) depending on its `has_error` state; the host detects the arm by
/// whether `close_button` is non-empty, and `instruction` then holds the error
/// text. `purge_button` is delivered but the macOS host has no widget for it.
/// Names are the engine DWH-getter symbols (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// +0x04: command sub-selector (1..0x7c; semantics unresolved).
    pub selector: u32,
    pub title: Ustring,
    pub instruction: Ustring,
    pub confirm_button: Ustring,
    pub cancel_button: Ustring,
    pub block_button: Ustring,
    pub close_button: Ustring,
    pub purge_button: Ustring,
}
