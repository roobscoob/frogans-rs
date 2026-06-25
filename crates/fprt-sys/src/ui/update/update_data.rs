//! `command_update_data` payload (`0x28`) — the dialog's two URIs.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The two URI strings the software-update dialog carries (distinct from its
/// labels). Names match the host `_strUpdateURI` / `_strChangedBranchURI` ivars
/// and the engine dwh setters (PROVEN).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateData {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → update_uri aligns to +0x08.
    pub update_uri: Ustring,
    pub changed_branch_uri: Ustring,
}
