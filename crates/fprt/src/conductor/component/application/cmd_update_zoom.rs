//! `update_zoom` command (engine → host).

use fprt_sys::ui::application::update_zoom::UpdateZoom as Raw;
use fprt_sys::ui::application::CMD_UPDATE_ZOOM;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

/// Scalar-only payload — carries no pool.
#[derive(Debug)]
pub struct UpdateZoom {
    /// Zoom level in percent (`100` = 100%).
    pub percent: i32,
}

impl CommandPayload for UpdateZoom {
    const ID: StatusName = CMD_UPDATE_ZOOM;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_zoom
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateZoom {
            percent: raw.zoom_level_percent,
        }
    }

    fn into_command(self) -> Command {
        Command::ApplicationUpdateZoom(self)
    }
}
