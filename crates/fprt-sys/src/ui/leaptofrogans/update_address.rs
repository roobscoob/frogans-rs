//! `command_update_address` payload (`0x20`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The candidate Frogans address being evaluated, plus a compliance flag the
/// host uses to choose which buttons to show.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateAddress {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    pub _rsv04: u32,
    /// The Frogans address text.
    pub address: Ustring,
    /// 1 = compliant, 0 = non-compliant (= `!has_error`).
    pub compliant_address: u32,
    pub _rsv1c: u32,
}
