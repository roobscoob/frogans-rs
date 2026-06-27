//! `update_labels` command (engine → host) — five **pooled** dialog strings.

use fprt_sys::ui::zoom::CMD_UPDATE_LABELS;
use fprt_sys::ui::zoom::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The five localized strings of the zoom-settings dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Default-button label.
    pub default_button: Option<PooledString>,
    /// Restore-button label (no separate button on the macOS host).
    pub restore_button: Option<PooledString>,
    /// OK-button label.
    pub ok_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        default_button: &str,
        restore_button: &str,
        ok_button: &str,
        cancel_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            default_button: Some(pool.alloc_str(default_button)),
            restore_button: Some(pool.alloc_str(restore_button)),
            ok_button: Some(pool.alloc_str(ok_button)),
            cancel_button: Some(pool.alloc_str(cancel_button)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                default_button: pool.string(raw.default_button),
                restore_button: pool.string(raw.restore_button),
                ok_button: pool.string(raw.ok_button),
                cancel_button: pool.string(raw.cancel_button),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            title: ustring_opt(self.title.as_ref()),
            default_button: ustring_opt(self.default_button.as_ref()),
            restore_button: ustring_opt(self.restore_button.as_ref()),
            ok_button: ustring_opt(self.ok_button.as_ref()),
            cancel_button: ustring_opt(self.cancel_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            default_button: pool.clone_str_opt(&self.default_button),
            restore_button: pool.clone_str_opt(&self.restore_button),
            ok_button: pool.clone_str_opt(&self.ok_button),
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
        let cmd = UpdateLabels::new(&pool, "Zoom", "Default", "Restore", "OK", "Cancel");
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Zoom");
        assert_eq!(back.default_button.unwrap().as_str().unwrap(), "Default");
        assert_eq!(back.cancel_button.unwrap().as_str().unwrap(), "Cancel");
    }
}
