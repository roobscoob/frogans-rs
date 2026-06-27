//! `update_labels` command (engine → host) — six **pooled** dialog strings.

use fprt_sys::ui::recentlyvisited::CMD_UPDATE_LABELS;
use fprt_sys::ui::recentlyvisited::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The six localized strings of the recently-visited dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Address-field placeholder.
    pub placeholder: Option<PooledString>,
    /// Open-button label.
    pub open_button: Option<PooledString>,
    /// Delete-button label.
    pub delete_button: Option<PooledString>,
    /// Delete-all-button label.
    pub delete_all_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        placeholder: &str,
        open_button: &str,
        delete_button: &str,
        delete_all_button: &str,
        cancel_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            placeholder: Some(pool.alloc_str(placeholder)),
            open_button: Some(pool.alloc_str(open_button)),
            delete_button: Some(pool.alloc_str(delete_button)),
            delete_all_button: Some(pool.alloc_str(delete_all_button)),
            cancel_button: Some(pool.alloc_str(cancel_button)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                placeholder: pool.string(raw.placeholder),
                open_button: pool.string(raw.open_button),
                delete_button: pool.string(raw.delete_button),
                delete_all_button: pool.string(raw.delete_all_button),
                cancel_button: pool.string(raw.cancel_button),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            title: ustring_opt(self.title.as_ref()),
            placeholder: ustring_opt(self.placeholder.as_ref()),
            open_button: ustring_opt(self.open_button.as_ref()),
            delete_button: ustring_opt(self.delete_button.as_ref()),
            delete_all_button: ustring_opt(self.delete_all_button.as_ref()),
            cancel_button: ustring_opt(self.cancel_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            placeholder: pool.clone_str_opt(&self.placeholder),
            open_button: pool.clone_str_opt(&self.open_button),
            delete_button: pool.clone_str_opt(&self.delete_button),
            delete_all_button: pool.clone_str_opt(&self.delete_all_button),
            cancel_button: pool.clone_str_opt(&self.cancel_button),
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
            &pool, "Recently visited", "frogans*…", "Open", "Delete", "Delete all", "Cancel",
        );
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Recently visited");
        assert_eq!(back.placeholder.unwrap().as_str().unwrap(), "frogans*…");
        assert_eq!(back.delete_all_button.unwrap().as_str().unwrap(), "Delete all");
        assert_eq!(back.cancel_button.unwrap().as_str().unwrap(), "Cancel");
    }
}
