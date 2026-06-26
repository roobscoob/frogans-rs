//! `update_data` command (engine → host) — the dialog's two URIs.

use fprt_sys::ui::update::update_data::UpdateData as Raw;
use fprt_sys::ui::update::CMD_UPDATE_DATA;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The two URIs the software-update dialog carries.
#[derive(Debug)]
pub struct UpdateData {
    /// The update URI.
    pub update_uri: Option<PooledString>,
    /// The changed-branch URI.
    pub changed_branch_uri: Option<PooledString>,
}

impl CommandPayload for UpdateData {
    const ID: StatusName = CMD_UPDATE_DATA;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.update_update_data
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateData {
                update_uri: pool.string(raw.update_uri),
                changed_branch_uri: pool.string(raw.changed_branch_uri),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::UpdateUpdateData(self)
    }
}
