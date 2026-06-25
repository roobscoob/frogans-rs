//! `command_update_directionality` payload (`0x08`).

use crate::ui::StatusName;
use crate::ui::application::directionality::Directionality;

/// Tells the host the text directionality enum.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateDirectionality {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    pub directionality: Directionality,
}
