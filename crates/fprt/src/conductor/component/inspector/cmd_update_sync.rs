//! `update_sync` command (engine → host) — autosync state + synchronize-enable.

use fprt_sys::ui::inspector::update_sync::UpdateSync as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_SYNC;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::Pool;

/// The inspector's auto-sync state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateSync {
    /// The target window.
    pub id: InspectorId,
    /// Autosync button polarity: `true` = ON (`0xbb9`), `false` = OFF (`0xbba`).
    pub autosync_on: bool,
    /// Whether the Synchronize button is enabled.
    pub synchronize_enabled: bool,
}

impl CommandPayload for UpdateSync {
    const ID: StatusName = CMD_UPDATE_SYNC;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_sync
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateSync {
            id: InspectorId(raw.reference),
            autosync_on: raw.autosync_mode == 0xbb9,
            synchronize_enabled: raw.synchronize_enabled != 0,
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateSync(self)
    }
}
