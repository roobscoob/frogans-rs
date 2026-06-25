//! `command_update_labels` payload (`0x68`) — six dialog strings.

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// The six localized UI strings of the favorites dialog.
///
/// `[?]` The six sub-labels are not individually recovered in available sources.
/// Its proven twin `recentlyvisited` has title / placeholder / open / delete /
/// delete-all / cancel — favorites is the same shape with *remove* in place of
/// *delete* — but rather than invent names they're kept as an array until a
/// favorites datatypes doc lands.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Labels {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    // +0x04: implicit pad → labels align to +0x08.
    pub labels: [Ustring; 6],
}
