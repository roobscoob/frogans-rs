//! `update_content_viewer` command (engine → host) — a document + syntax mode.

use fprt_sys::ui::inspector::update_content_viewer::UpdateContentViewer as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_CONTENT_VIEWER;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::{Pool, PooledString};

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
    fn from_raw(raw: u32) -> Self {
        match raw {
            0x7d1 => ContentMode::Json,
            0x7d2 => ContentMode::Xml,
            0x7d3 => ContentMode::Uxce,
            _ => ContentMode::Other,
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

impl CommandPayload for UpdateContentViewer {
    const ID: StatusName = CMD_UPDATE_CONTENT_VIEWER;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_content_viewer
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `content` was written into `pool` by the pop that produced both.
        let content = unsafe { pool.string(raw.content) };
        UpdateContentViewer {
            id: InspectorId(raw.reference),
            content_mode: ContentMode::from_raw(raw.content_mode),
            content,
            content_select: raw.content_select,
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateContentViewer(self)
    }
}
