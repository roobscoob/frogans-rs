//! `command_update_legal_content` payload (`0x38`) + its nested content tree.

use crate::ui::{ImageRecord, StatusName};
use crate::ustring::Ustring;

/// One legal-document topic: a title + its HTML body. Host class `HtmlDoc`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HtmlDoc {
    pub title: Ustring,
    pub html_content: Ustring,
}

/// One per-language legal document: a language name + an array of topics. Host
/// class `LegalDoc`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LegalDoc {
    /// Language / section name.
    pub language: Ustring,
    /// Number of topics.
    pub topic_count: i32,
    pub _rsv14: u32,
    /// Mempool array of `topic_count` topics (stride `0x20`).
    pub topics: *const HtmlDoc,
}

/// The legal-information document content: a content-kind selector, an optional
/// image, a default-language index, and an array of per-language documents.
///
/// (No `Debug`: contains [`ImageRecord`], whose union has no `Debug`.)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UpdateLegalContent {
    /// Field 0 — engine-stamped status name.
    pub status_id: StatusName,
    /// `0x2329` text / `0x232a` image / `9000` other.
    pub content_kind: u32,
    /// Filled only when `content_kind == 0x232a` (image).
    pub image: ImageRecord,
    /// Default-language index into `docs`.
    pub default_language: u32,
    pub _rsv24: u32,
    /// Number of documents.
    pub doc_count: i32,
    // +0x2c: implicit pad → docs aligns to +0x30.
    /// Mempool array of `doc_count` documents (stride `0x20`).
    pub docs: *const LegalDoc,
}
