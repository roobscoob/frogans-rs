//! `update_status` command (engine → host) — run outcome + data-available flag.

use fprt_sys::ui::inspector::update_status::UpdateStatus as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_STATUS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::Pool;

/// The outcome of a site run, as the inspector should display it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunStatus {
    /// Run completed (`0x3e9`).
    Completed,
    /// A run rejection was raised (`0x3ea`).
    RejectionRaised,
    /// An engine value outside the documented set.
    Other(u32),
}

impl RunStatus {
    fn from_raw(raw: u32) -> Self {
        match raw {
            0x3e9 => RunStatus::Completed,
            0x3ea => RunStatus::RejectionRaised,
            other => RunStatus::Other(other),
        }
    }
}

/// The inspector's run status plus whether run data is available to show.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateStatus {
    /// The target window.
    pub id: InspectorId,
    /// The run outcome.
    pub run_status: RunStatus,
    /// `false` ⇒ show "run data not available".
    pub run_data_available: bool,
}

impl CommandPayload for UpdateStatus {
    const ID: StatusName = CMD_UPDATE_STATUS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_status
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateStatus {
            id: InspectorId(raw.reference),
            run_status: RunStatus::from_raw(raw.run_status),
            run_data_available: raw.run_data_available != 0,
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateStatus(self)
    }
}
