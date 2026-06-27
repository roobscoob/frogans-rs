//! `update_labels` command (engine → host) — the inspector's full text set
//! (ten **pooled** strings).

use fprt_sys::ui::inspector::CMD_UPDATE_LABELS;
use fprt_sys::ui::inspector::labels::Labels as Raw;

use crate::component::inspector::InspectorId;
use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

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

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: &OwnedPool,
        id: InspectorId,
        title: &str,
        run_completed: &str,
        run_rejection_raised: &str,
        synchronize_button: &str,
        rerun_button_reload: &str,
        rerun_button_retry: &str,
        run_data_not_available: &str,
        autosync_button_on: &str,
        autosync_button_off: &str,
        close_button: &str,
    ) -> Self {
        UpdateLabels {
            id,
            title: Some(pool.alloc_str(title)),
            run_completed: Some(pool.alloc_str(run_completed)),
            run_rejection_raised: Some(pool.alloc_str(run_rejection_raised)),
            synchronize_button: Some(pool.alloc_str(synchronize_button)),
            rerun_button_reload: Some(pool.alloc_str(rerun_button_reload)),
            rerun_button_retry: Some(pool.alloc_str(rerun_button_retry)),
            run_data_not_available: Some(pool.alloc_str(run_data_not_available)),
            autosync_button_on: Some(pool.alloc_str(autosync_button_on)),
            autosync_button_off: Some(pool.alloc_str(autosync_button_off)),
            close_button: Some(pool.alloc_str(close_button)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
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

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            reference: self.id.0,
            title: ustring_opt(self.title.as_ref()),
            run_completed: ustring_opt(self.run_completed.as_ref()),
            run_rejection_raised: ustring_opt(self.run_rejection_raised.as_ref()),
            synchronize_button: ustring_opt(self.synchronize_button.as_ref()),
            rerun_button_reload: ustring_opt(self.rerun_button_reload.as_ref()),
            rerun_button_retry: ustring_opt(self.rerun_button_retry.as_ref()),
            run_data_not_available: ustring_opt(self.run_data_not_available.as_ref()),
            autosync_button_on: ustring_opt(self.autosync_button_on.as_ref()),
            autosync_button_off: ustring_opt(self.autosync_button_off.as_ref()),
            close_button: ustring_opt(self.close_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            id: self.id,
            title: pool.clone_str_opt(&self.title),
            run_completed: pool.clone_str_opt(&self.run_completed),
            run_rejection_raised: pool.clone_str_opt(&self.run_rejection_raised),
            synchronize_button: pool.clone_str_opt(&self.synchronize_button),
            rerun_button_reload: pool.clone_str_opt(&self.rerun_button_reload),
            rerun_button_retry: pool.clone_str_opt(&self.rerun_button_retry),
            run_data_not_available: pool.clone_str_opt(&self.run_data_not_available),
            autosync_button_on: pool.clone_str_opt(&self.autosync_button_on),
            autosync_button_off: pool.clone_str_opt(&self.autosync_button_off),
            close_button: pool.clone_str_opt(&self.close_button),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateLabels::new(
            &pool,
            InspectorId(9),
            "Inspector",
            "Run completed",
            "Run rejection raised",
            "Synchronize",
            "Reload",
            "Retry",
            "Run data not available",
            "Autosync on",
            "Autosync off",
            "Close",
        );
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.id, InspectorId(9));
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Inspector");
        assert_eq!(back.rerun_button_retry.unwrap().as_str().unwrap(), "Retry");
        assert_eq!(back.close_button.unwrap().as_str().unwrap(), "Close");
    }
}
