//! `update_labels` command (engine → host) — four **pooled** dialog strings.

use fprt_sys::ui::devtools::CMD_UPDATE_LABELS;
use fprt_sys::ui::devtools::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The four localized strings of the developers-directory (DevTools) dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Address-field placeholder.
    pub placeholder: Option<PooledString>,
    /// Inspect-button label.
    pub inspect_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        placeholder: &str,
        inspect_button: &str,
        cancel_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            placeholder: Some(pool.alloc_str(placeholder)),
            inspect_button: Some(pool.alloc_str(inspect_button)),
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
                inspect_button: pool.string(raw.inspect_button),
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
            inspect_button: ustring_opt(self.inspect_button.as_ref()),
            cancel_button: ustring_opt(self.cancel_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            placeholder: pool.clone_str_opt(&self.placeholder),
            inspect_button: pool.clone_str_opt(&self.inspect_button),
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
        let cmd = UpdateLabels::new(&pool, "DevTools", "frogans*…", "Inspect", "Cancel");
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "DevTools");
        assert_eq!(back.placeholder.unwrap().as_str().unwrap(), "frogans*…");
        assert_eq!(back.inspect_button.unwrap().as_str().unwrap(), "Inspect");
        assert_eq!(back.cancel_button.unwrap().as_str().unwrap(), "Cancel");
    }
}
