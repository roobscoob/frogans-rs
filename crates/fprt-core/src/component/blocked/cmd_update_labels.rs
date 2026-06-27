//! `update_labels` command (engine → host) — five **pooled** dialog strings.

use fprt_sys::ui::blocked::CMD_UPDATE_LABELS;
use fprt_sys::ui::blocked::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The five localized strings of the blocked-addresses dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Address-field placeholder.
    pub placeholder: Option<PooledString>,
    /// Close-button label.
    pub close_button: Option<PooledString>,
    /// Remove-button label.
    pub remove_button: Option<PooledString>,
    /// Remove-all-button label.
    pub remove_all_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        placeholder: &str,
        close_button: &str,
        remove_button: &str,
        remove_all_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            placeholder: Some(pool.alloc_str(placeholder)),
            close_button: Some(pool.alloc_str(close_button)),
            remove_button: Some(pool.alloc_str(remove_button)),
            remove_all_button: Some(pool.alloc_str(remove_all_button)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                placeholder: pool.string(raw.placeholder),
                close_button: pool.string(raw.close_button),
                remove_button: pool.string(raw.remove_button),
                remove_all_button: pool.string(raw.remove_all_button),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            title: ustring_opt(self.title.as_ref()),
            placeholder: ustring_opt(self.placeholder.as_ref()),
            close_button: ustring_opt(self.close_button.as_ref()),
            remove_button: ustring_opt(self.remove_button.as_ref()),
            remove_all_button: ustring_opt(self.remove_all_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            placeholder: pool.clone_str_opt(&self.placeholder),
            close_button: pool.clone_str_opt(&self.close_button),
            remove_button: pool.clone_str_opt(&self.remove_button),
            remove_all_button: pool.clone_str_opt(&self.remove_all_button),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateLabels::new(&pool, "Blocked", "frogans*…", "Close", "Remove", "Remove all");
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Blocked");
        assert_eq!(back.placeholder.unwrap().as_str().unwrap(), "frogans*…");
        assert_eq!(back.remove_all_button.unwrap().as_str().unwrap(), "Remove all");
    }
}
