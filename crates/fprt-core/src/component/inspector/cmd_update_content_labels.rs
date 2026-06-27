//! `update_content_labels` command (engine → host) — the content selector
//! (a **pooled** label list + the active index).

use fprt_sys::ui::inspector::CMD_UPDATE_CONTENT_LABELS;
use fprt_sys::ui::inspector::update_content_labels::UpdateContentLabels as Raw;
use fprt_sys::ustring::Ustring;

use crate::component::inspector::InspectorId;
use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The inspector content-selector entries plus the active index.
#[derive(Debug)]
pub struct UpdateContentLabels {
    /// The target window.
    pub id: InspectorId,
    /// The content labels, in engine order.
    pub labels: Vec<PooledString>,
    /// Index into `labels` that is selected/active.
    pub content_active: i32,
}

impl UpdateContentLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(pool: &OwnedPool, id: InspectorId, labels: &[&str], content_active: i32) -> Self {
        UpdateContentLabels {
            id,
            labels: labels.iter().map(|s| pool.alloc_str(s)).collect(),
            content_active,
        }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `labels` points at `count` entries written into `pool` by the pop.
        UpdateContentLabels {
            id: InspectorId(raw.reference),
            labels: unsafe { pool.strings(raw.labels, raw.count) },
            content_active: raw.content_active,
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
            status_id: CMD_UPDATE_CONTENT_LABELS,
            reference: self.id.0,
            count,
            _rsv0c: 0,
            labels,
            content_active: self.content_active,
            _rsv1c: 0,
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateContentLabels {
            id: self.id,
            labels: self.labels.iter().map(|s| pool.clone_str(s)).collect(),
            content_active: self.content_active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd =
            UpdateContentLabels::new(&pool, InspectorId(4), &["request", "response", "trace"], 2);
        let back = UpdateContentLabels::from_raw(cmd.to_raw(&pool), &pool.as_pool());
        assert_eq!(back.id, InspectorId(4));
        assert_eq!(back.content_active, 2);
        let got: Vec<&str> = back.labels.iter().map(|s| s.as_str().unwrap()).collect();
        assert_eq!(got, ["request", "response", "trace"]);
    }

    #[test]
    fn empty_list_encodes_null() {
        let pool = OwnedPool::new();
        let raw = UpdateContentLabels {
            id: InspectorId(0),
            labels: vec![],
            content_active: 0,
        }
        .to_raw(&pool);
        assert!(raw.labels.is_null());
        assert_eq!(raw.count, 0);
    }
}
