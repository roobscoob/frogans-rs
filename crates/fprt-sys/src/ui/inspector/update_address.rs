//! `command_update_address` payload (`0x18`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The Frogans address text shown in the inspector's address field.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateAddress {
    pub status_id: StatusName,
    pub reference: i32,
    pub address: Ustring,
}
