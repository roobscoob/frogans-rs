//! `command_launch_way_out` payload (`0x18`).

use crate::ui::StatusName;
use crate::ui::application::uri_scheme::UriScheme;
use crate::ustring::Ustring;

/// A URL the host must open externally (browser / mail client) — the
/// string-carrying command.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LaunchWayOut {
    /// Field 0 — `StatusName::NONE` (empty) / `StatusName::FALLBACK` (url present).
    pub type_tag: StatusName,
    pub uri_scheme: UriScheme,
    /// The URL (`utf8` is mempool-owned — free via the call's `mempool_out`).
    pub uri: Ustring,
}
