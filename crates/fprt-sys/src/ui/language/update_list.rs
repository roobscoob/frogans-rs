//! `command_update_list` payload (`0x28`) + its entry record (`0x20`).

use crate::ui::StatusName;
use crate::ustring::Ustring;

/// One selectable interface language.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LangEntry {
    /// Language code/identifier (BCP-47 tag vs internal key — form unresolved).
    pub identifier: Ustring,
    /// Human-readable display name.
    pub language: Ustring,
}

/// The list of selectable languages plus the intended current selection.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UpdateList {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    pub _rsv04: u32,
    /// Number of entries.
    pub count: u32,
    // +0x0c: implicit pad → entries aligns to +0x10.
    /// Mempool array of `count` entries (stride `0x20`).
    pub entries: *const LangEntry,
    /// Intended current selection id (macOS engine leaves it empty; host reads it).
    pub current_lang_id: Ustring,
}
