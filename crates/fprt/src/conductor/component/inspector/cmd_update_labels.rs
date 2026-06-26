//! `update_labels` command (engine → host) — the inspector's full text set.

use fprt_sys::ui::inspector::labels::Labels as Raw;
use fprt_sys::ui::inspector::CMD_UPDATE_LABELS;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::component::inspector::InspectorId;
use crate::pool::{Pool, PooledString};

/// The inspector window's ten localized strings.
#[derive(Debug)]
pub struct UpdateLabels {
    /// The target window.
    pub id: InspectorId,
    /// Window title.
    pub title: Option<PooledString>,
    /// "Run completed" status text.
    pub run_completed: Option<PooledString>,
    /// "Run rejection raised" status text.
    pub run_rejection_raised: Option<PooledString>,
    /// Synchronize-button label.
    pub synchronize_button: Option<PooledString>,
    /// Rerun-button label in "reload" polarity.
    pub rerun_button_reload: Option<PooledString>,
    /// Rerun-button label in "retry" polarity.
    pub rerun_button_retry: Option<PooledString>,
    /// "Run data not available" text.
    pub run_data_not_available: Option<PooledString>,
    /// Autosync-button label when on.
    pub autosync_button_on: Option<PooledString>,
    /// Autosync-button label when off.
    pub autosync_button_off: Option<PooledString>,
    /// Close-button label.
    pub close_button: Option<PooledString>,
}

impl CommandPayload for UpdateLabels {
    const ID: StatusName = CMD_UPDATE_LABELS;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.inspector_update_labels
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every string was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                id: InspectorId(raw.reference),
                title: pool.string(raw.title),
                run_completed: pool.string(raw.run_completed),
                run_rejection_raised: pool.string(raw.run_rejection_raised),
                synchronize_button: pool.string(raw.synchronize_button),
                rerun_button_reload: pool.string(raw.rerun_button_reload),
                rerun_button_retry: pool.string(raw.rerun_button_retry),
                run_data_not_available: pool.string(raw.run_data_not_available),
                autosync_button_on: pool.string(raw.autosync_button_on),
                autosync_button_off: pool.string(raw.autosync_button_off),
                close_button: pool.string(raw.close_button),
            }
        }
    }

    fn into_command(self) -> Command {
        Command::InspectorUpdateLabels(self)
    }
}
