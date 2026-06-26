//! `update_legal_content` command (engine → host) — the nested document tree.

use fprt_sys::ui::legalinformation::legal_content::UpdateLegalContent as Raw;
use fprt_sys::ui::legalinformation::CMD_UPDATE_LEGAL_CONTENT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledImage, PooledString};

/// What kind of content the panel should render.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegalContentKind {
    /// Per-language HTML documents (`0x2329`).
    Text,
    /// A single image (`0x232a`) — see [`UpdateLegalContent::image`].
    Image,
    /// An engine value outside the documented set (`9000` or other).
    Other,
}

impl LegalContentKind {
    fn from_raw(raw: u32) -> Self {
        match raw {
            0x2329 => LegalContentKind::Text,
            0x232a => LegalContentKind::Image,
            _ => LegalContentKind::Other,
        }
    }
}

/// One legal-document topic: a title and its HTML body.
#[derive(Debug)]
pub struct Topic {
    /// Topic title.
    pub title: Option<PooledString>,
    /// HTML body.
    pub html: Option<PooledString>,
}

/// One per-language legal document: a language name and its topics.
#[derive(Debug)]
pub struct Document {
    /// Language / section name.
    pub language: Option<PooledString>,
    /// The document's topics, in engine order.
    pub topics: Vec<Topic>,
}

/// The legal-information panel's content: a kind selector, an optional image
/// (filled only when `kind` is [`Image`](LegalContentKind::Image)), the
/// default-language index, and the per-language documents.
#[derive(Debug)]
pub struct UpdateLegalContent {
    /// Which rendering path the panel should take.
    pub kind: LegalContentKind,
    /// The image, when `kind` is [`Image`](LegalContentKind::Image).
    pub image: Option<PooledImage>,
    /// Default-language index into `documents`.
    pub default_language: u32,
    /// The per-language documents, in engine order.
    pub documents: Vec<Document>,
}

impl CommandPayload for UpdateLegalContent {
    const ID: StatusName = CMD_UPDATE_LEGAL_CONTENT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.legalinformation_update_legal_content
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        let mut documents = Vec::with_capacity(raw.doc_count.max(0) as usize);
        if !raw.docs.is_null() {
            for i in 0..raw.doc_count.max(0) as usize {
                // SAFETY: `docs` points at `doc_count` records (stride 0x20), all
                // written into `pool` by the pop that produced them.
                let doc = unsafe { *raw.docs.add(i) };
                let mut topics = Vec::with_capacity(doc.topic_count.max(0) as usize);
                if !doc.topics.is_null() {
                    for j in 0..doc.topic_count.max(0) as usize {
                        // SAFETY: `topics` points at `topic_count` records (stride
                        // 0x20) from the same pool.
                        let topic = unsafe { *doc.topics.add(j) };
                        let (title, html) =
                            unsafe { (pool.string(topic.title), pool.string(topic.html_content)) };
                        topics.push(Topic { title, html });
                    }
                }
                // SAFETY: `language` was written into `pool` by the same pop.
                let language = unsafe { pool.string(doc.language) };
                documents.push(Document { language, topics });
            }
        }
        // SAFETY: `image` borrows the same pool; only meaningful when kind == Image.
        let image = unsafe { pool.image(raw.image) };
        UpdateLegalContent {
            kind: LegalContentKind::from_raw(raw.content_kind),
            image,
            default_language: raw.default_language,
            documents,
        }
    }

    fn into_command(self) -> Command {
        Command::LegalinformationUpdateLegalContent(self)
    }
}
