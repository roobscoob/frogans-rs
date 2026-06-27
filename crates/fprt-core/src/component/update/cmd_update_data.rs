//! `update_data` command (engine → host) — the dialog's two **pooled** URIs.

use fprt_sys::ui::update::CMD_UPDATE_DATA;
use fprt_sys::ui::update::update_data::UpdateData as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The two URIs the software-update dialog carries.
#[derive(Debug)]
pub struct UpdateData {
    /// The update URI.
    pub update_uri: Option<PooledString>,
    /// The changed-branch URI.
    pub changed_branch_uri: Option<PooledString>,
}

impl UpdateData {
    /// Build one, allocating each URI into `pool`.
    pub fn new(pool: &OwnedPool, update_uri: &str, changed_branch_uri: &str) -> Self {
        UpdateData {
            update_uri: Some(pool.alloc_str(update_uri)),
            changed_branch_uri: Some(pool.alloc_str(changed_branch_uri)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateData {
                update_uri: pool.string(raw.update_uri),
                changed_branch_uri: pool.string(raw.changed_branch_uri),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_DATA,
            update_uri: ustring_opt(self.update_uri.as_ref()),
            changed_branch_uri: ustring_opt(self.changed_branch_uri.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateData {
            update_uri: pool.clone_str_opt(&self.update_uri),
            changed_branch_uri: pool.clone_str_opt(&self.changed_branch_uri),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateData::new(
            &pool,
            "https://example.org/update",
            "https://example.org/branch",
        );
        let back = UpdateData::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(
            back.update_uri.unwrap().as_str().unwrap(),
            "https://example.org/update"
        );
        assert_eq!(
            back.changed_branch_uri.unwrap().as_str().unwrap(),
            "https://example.org/branch"
        );
    }
}
