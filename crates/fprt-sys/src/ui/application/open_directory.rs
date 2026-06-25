//! `command_open_directory` payload (`0x08`).

use crate::ui::StatusName;
use crate::ui::application::open_dir_kind::OpenDirKind;

/// Tells the host to reveal a known directory (the developers dir) in the file
/// manager. No path string is carried — the host knows the path.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OpenDirectory {
    /// Field 0 — directory-identity id (driven by the command reference).
    /// Field name **[unresolved]**.
    pub dir_id: StatusName,
    pub kind: OpenDirKind,
}
