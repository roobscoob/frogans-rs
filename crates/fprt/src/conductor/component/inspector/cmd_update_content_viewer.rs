//! `update_content_viewer` command (engine → host) — a document + syntax mode.

use fprt_sys::ui::inspector::update_content_viewer::UpdateContentViewer as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_CONTENT_VIEWER;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::inspector::{ContentMode, UpdateContentViewer};

impl CommandPayload for UpdateContentViewer {
    const ID: StatusName = CMD_UPDATE_CONTENT_VIEWER;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_content_viewer
    }

    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::InspectorUpdateContentViewer(UpdateContentViewer::from_raw(raw, pool))
    }
}
