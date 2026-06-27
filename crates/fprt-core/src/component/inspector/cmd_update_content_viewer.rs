//! `update_content_viewer` command (engine → host) — a document (**pooled**
//! string) plus its syntax mode and a trailing selector int.

use fprt_sys::ui::inspector::CMD_UPDATE_CONTENT_VIEWER;
use fprt_sys::ui::inspector::update_content_viewer::UpdateContentViewer as Raw;

use crate::component::inspector::InspectorId;
use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The syntax-highlighting mode for the content viewer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContentMode {
    /// JSON (`0x7d1`).
    Json,
    /// XML (`0x7d2`).
    Xml,
    /// UXCE (`0x7d3`).
    Uxce,
    /// Default / binary / unknown (`0x7d0` or any other value).
    Other,
}

impl ContentMode {
    /// Map the raw content-mode word.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0x7d1 => ContentMode::Json,
            0x7d2 => ContentMode::Xml,
            0x7d3 => ContentMode::Uxce,
            _ => ContentMode::Other,
        }
    }

    /// Map back to the raw content-mode word ([`Other`](ContentMode::Other) ⇒
    /// the default `0x7d0`).
    pub fn to_raw(self) -> u32 {
        match self {
            ContentMode::Json => 0x7d1,
            ContentMode::Xml => 0x7d2,
            ContentMode::Uxce => 0x7d3,
            ContentMode::Other => 0x7d0,
        }
    }
}

/// One content document loaded into the inspector's viewer.
#[derive(Debug)]
pub struct UpdateContentViewer {
    /// The target window.
    pub id: InspectorId,
    /// The syntax mode to render with.
    pub content_mode: ContentMode,
    /// Document text.
    pub content: Option<PooledString>,
    /// Trailing selector/position int (engine emits 0 / -1).
    pub content_select: i32,
}

impl UpdateContentViewer {
    /// Build one, allocating `content` into `pool`.
    pub fn new(
        pool: &OwnedPool,
        id: InspectorId,
        content_mode: ContentMode,
        content: &str,
        content_select: i32,
    ) -> Self {
        UpdateContentViewer {
            id,
            content_mode,
            content: Some(pool.alloc_str(content)),
            content_select,
        }
    }

    /// Decode the engine's payload, wrapping its pooled bytes zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `content` was written into `pool` by the pop that produced both.
        UpdateContentViewer {
            id: InspectorId(raw.reference),
            content_mode: ContentMode::from_raw(raw.content_mode),
            content: unsafe { pool.string(raw.content) },
            content_select: raw.content_select,
        }
    }

    /// Encode into the raw payload (descriptor points at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_CONTENT_VIEWER,
            reference: self.id.0,
            content_mode: self.content_mode.to_raw(),
            _rsv0c: 0,
            content: ustring_opt(self.content.as_ref()),
            content_select: self.content_select,
            _rsv24: 0,
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateContentViewer {
            id: self.id,
            content_mode: self.content_mode,
            content: pool.clone_str_opt(&self.content),
            content_select: self.content_select,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        for mode in [
            ContentMode::Json,
            ContentMode::Xml,
            ContentMode::Uxce,
            ContentMode::Other,
        ] {
            let cmd = UpdateContentViewer::new(&pool, InspectorId(1), mode, "{\"k\":1}", -1);
            let back = UpdateContentViewer::from_raw(cmd.to_raw(), &pool.as_pool());
            assert_eq!(back.id, InspectorId(1));
            assert_eq!(back.content_mode, mode);
            assert_eq!(back.content_select, -1);
            assert_eq!(back.content.unwrap().as_str().unwrap(), "{\"k\":1}");
        }
    }
}
