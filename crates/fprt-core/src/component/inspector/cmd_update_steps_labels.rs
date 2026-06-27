//! `update_steps_labels` command (engine → host) — the run-step combobox
//! (a **pooled** label list + the active index).

use fprt_sys::ui::inspector::CMD_UPDATE_STEPS_LABELS;
use fprt_sys::ui::inspector::update_steps_labels::UpdateStepsLabels as Raw;
use fprt_sys::ustring::Ustring;

use crate::component::inspector::InspectorId;
use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The inspector step-combobox entries plus the index to pre-select.
#[derive(Debug)]
pub struct UpdateStepsLabels {
    /// The target window.
    pub id: InspectorId,
    /// The step labels, in engine order.
    pub labels: Vec<PooledString>,
    /// Index into `labels` to pre-select.
    pub active_step: i32,
}

impl UpdateStepsLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(pool: &OwnedPool, id: InspectorId, labels: &[&str], active_step: i32) -> Self {
        UpdateStepsLabels {
            id,
            labels: labels.iter().map(|s| pool.alloc_str(s)).collect(),
            active_step,
        }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `labels` points at `count` entries written into `pool` by the pop.
        UpdateStepsLabels {
            id: InspectorId(raw.reference),
            labels: unsafe { pool.strings(raw.labels, raw.count) },
            active_step: raw.active_step,
        }
    }

    /// Encode into the raw payload, allocating the descriptor array into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        let (count, labels) = if self.labels.is_empty() {
            (0, core::ptr::null())
        } else {
            let descriptors: Vec<Ustring> =
                self.labels.iter().map(|s| ustring_opt(Some(s))).collect();
            (
                self.labels.len() as u32,
                pool.alloc_slice(&descriptors).cast::<Ustring>(),
            )
        };
        Raw {
            status_id: CMD_UPDATE_STEPS_LABELS,
            reference: self.id.0,
            count,
            _rsv0c: 0,
            labels,
            active_step: self.active_step,
            _rsv1c: 0,
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateStepsLabels {
            id: self.id,
            labels: self.labels.iter().map(|s| pool.clone_str(s)).collect(),
            active_step: self.active_step,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateStepsLabels::new(&pool, InspectorId(2), &["resolve", "fetch", "render"], 1);
        let back = UpdateStepsLabels::from_raw(cmd.to_raw(&pool), &pool.as_pool());
        assert_eq!(back.id, InspectorId(2));
        assert_eq!(back.active_step, 1);
        let got: Vec<&str> = back.labels.iter().map(|s| s.as_str().unwrap()).collect();
        assert_eq!(got, ["resolve", "fetch", "render"]);
    }

    #[test]
    fn empty_list_encodes_null() {
        let pool = OwnedPool::new();
        let raw = UpdateStepsLabels {
            id: InspectorId(0),
            labels: vec![],
            active_step: 0,
        }
        .to_raw(&pool);
        assert!(raw.labels.is_null());
        assert_eq!(raw.count, 0);
    }
}
