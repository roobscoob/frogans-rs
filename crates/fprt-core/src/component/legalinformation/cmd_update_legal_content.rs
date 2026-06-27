//! `update_legal_content` command (engine → host) — the nested document tree.
//!
//! The engine produces this nested content tree (per-language documents, each
//! with a topic list, plus an optional image), and the host reads it back
//! (`from_raw`). The encode side (`new`/`to_raw`) builds the document/topic
//! arrays into a pool, allocating the inner topic arrays first and then the
//! outer document array that points at them.

use fprt_sys::ui::legalinformation::CMD_UPDATE_LEGAL_CONTENT;
use fprt_sys::ui::legalinformation::legal_content::{
    HtmlDoc, LegalDoc, UpdateLegalContent as Raw,
};

use crate::pool::{OwnedPool, Pool, PooledImage, PooledString};
use crate::wire::{image_record, ustring_opt};

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
    /// Map the raw content-kind word.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0x2329 => LegalContentKind::Text,
            0x232a => LegalContentKind::Image,
            _ => LegalContentKind::Other,
        }
    }

    /// The inverse of [`from_raw`](Self::from_raw): the engine word for this
    /// kind. [`Other`](Self::Other) encodes as the documented `9000` fallback.
    pub fn to_raw(self) -> u32 {
        match self {
            LegalContentKind::Text => 0x2329,
            LegalContentKind::Image => 0x232a,
            LegalContentKind::Other => 9000,
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

impl Topic {
    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Topic {
            title: pool.clone_str_opt(&self.title),
            html: pool.clone_str_opt(&self.html),
        }
    }
}

/// One per-language legal document: a language name and its topics.
#[derive(Debug)]
pub struct Document {
    /// Language / section name.
    pub language: Option<PooledString>,
    /// The document's topics, in engine order.
    pub topics: Vec<Topic>,
}

impl Document {
    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Document {
            language: pool.clone_str_opt(&self.language),
            topics: self.topics.iter().map(|t| t.copy_into(pool)).collect(),
        }
    }
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

impl UpdateLegalContent {
    /// Build a [`Text`](LegalContentKind::Text) content tree, allocating every
    /// language name, topic title and topic body into `pool`.
    ///
    /// Each document is `(language_name, &[(topic_title, topic_html)])`. The
    /// resulting payload carries no image (`image: None`).
    pub fn new(
        pool: &OwnedPool,
        documents: &[(&str, &[(&str, &str)])],
        default_language: u32,
    ) -> Self {
        let documents = documents
            .iter()
            .map(|(language, topics)| Document {
                language: Some(pool.alloc_str(language)),
                topics: topics
                    .iter()
                    .map(|(title, html)| Topic {
                        title: Some(pool.alloc_str(title)),
                        html: Some(pool.alloc_str(html)),
                    })
                    .collect(),
            })
            .collect();
        UpdateLegalContent {
            kind: LegalContentKind::Text,
            image: None,
            default_language,
            documents,
        }
    }

    /// Decode the engine's nested content tree.
    ///
    /// # Safety
    /// `raw`'s document/topic arrays, strings, and image must all point into
    /// `pool` (i.e. `raw` was produced by the pop that owns `pool`).
    pub unsafe fn from_raw(raw: Raw, pool: &Pool) -> Self {
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

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLegalContent {
            kind: self.kind,
            image: pool.clone_image_opt(&self.image),
            default_language: self.default_language,
            documents: self.documents.iter().map(|d| d.copy_into(pool)).collect(),
        }
    }

    /// Encode into the raw payload, allocating the nested document/topic
    /// descriptor arrays into `pool`.
    ///
    /// The inner topic array of each document is allocated first; the resulting
    /// pointer + count is then captured into that document's [`LegalDoc`]
    /// record, and the outer document array is allocated last. Empty arrays
    /// encode as a null pointer with a zero count.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        let doc_records: Vec<LegalDoc> = self
            .documents
            .iter()
            .map(|doc| {
                let (topic_count, topics) = if doc.topics.is_empty() {
                    (0, core::ptr::null())
                } else {
                    let topic_records: Vec<HtmlDoc> = doc
                        .topics
                        .iter()
                        .map(|t| HtmlDoc {
                            title: ustring_opt(t.title.as_ref()),
                            html_content: ustring_opt(t.html.as_ref()),
                        })
                        .collect();
                    (
                        doc.topics.len() as i32,
                        pool.alloc_slice(&topic_records).cast::<HtmlDoc>(),
                    )
                };
                LegalDoc {
                    language: ustring_opt(doc.language.as_ref()),
                    topic_count,
                    _rsv14: 0,
                    topics,
                }
            })
            .collect();

        let (doc_count, docs) = if doc_records.is_empty() {
            (0, core::ptr::null())
        } else {
            (
                doc_records.len() as i32,
                pool.alloc_slice(&doc_records).cast::<LegalDoc>(),
            )
        };

        Raw {
            status_id: CMD_UPDATE_LEGAL_CONTENT,
            content_kind: self.kind.to_raw(),
            image: image_record(self.image.as_ref()),
            default_language: self.default_language,
            _rsv24: 0,
            doc_count,
            docs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateLegalContent::new(
            &pool,
            &[
                ("English", &[("Terms", "<p>terms</p>"), ("Privacy", "<p>privacy</p>")]),
                ("Français", &[("Mentions", "<p>mentions</p>")]),
            ],
            1,
        );
        // SAFETY: `to_raw` wrote every array/string into `pool`, which `as_pool`
        // keeps alive for the decode.
        let back = unsafe { UpdateLegalContent::from_raw(cmd.to_raw(&pool), &pool.as_pool()) };

        assert_eq!(back.kind, LegalContentKind::Text);
        assert_eq!(back.default_language, 1);
        assert_eq!(back.documents.len(), 2);

        let langs: Vec<&str> = back
            .documents
            .iter()
            .map(|d| d.language.as_ref().unwrap().as_str().unwrap())
            .collect();
        assert_eq!(langs, ["English", "Français"]);

        let titles: Vec<&str> = back.documents[0]
            .topics
            .iter()
            .map(|t| t.title.as_ref().unwrap().as_str().unwrap())
            .collect();
        assert_eq!(titles, ["Terms", "Privacy"]);

        let htmls: Vec<&str> = back.documents[0]
            .topics
            .iter()
            .map(|t| t.html.as_ref().unwrap().as_str().unwrap())
            .collect();
        assert_eq!(htmls, ["<p>terms</p>", "<p>privacy</p>"]);

        assert_eq!(back.documents[1].topics.len(), 1);
        assert_eq!(
            back.documents[1].topics[0].html.as_ref().unwrap().as_str().unwrap(),
            "<p>mentions</p>"
        );
    }

    #[test]
    fn empty_documents_encode_null() {
        let pool = OwnedPool::new();
        let raw = UpdateLegalContent::new(&pool, &[], 0).to_raw(&pool);
        assert!(raw.docs.is_null());
        assert_eq!(raw.doc_count, 0);
        assert_eq!(raw.content_kind, 0x2329);
    }
}
