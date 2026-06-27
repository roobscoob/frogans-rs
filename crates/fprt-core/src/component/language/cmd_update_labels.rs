//! `update_labels` command (engine → host) — five **pooled** dialog strings.

use fprt_sys::ui::language::CMD_UPDATE_LABELS;
use fprt_sys::ui::language::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The five localized strings of the language-selection dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// "Current language" label.
    pub current: Option<PooledString>,
    /// "Select" prompt label.
    pub select: Option<PooledString>,
    /// OK-button label.
    pub ok_button: Option<PooledString>,
    /// Cancel/Close-button label.
    pub cancel_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        current: &str,
        select: &str,
        ok_button: &str,
        cancel_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            current: Some(pool.alloc_str(current)),
            select: Some(pool.alloc_str(select)),
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
                current: pool.string(raw.current),
                select: pool.string(raw.select),
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
            current: ustring_opt(self.current.as_ref()),
            select: ustring_opt(self.select.as_ref()),
            ok_button: ustring_opt(self.ok_button.as_ref()),
            cancel_button: ustring_opt(self.cancel_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            current: pool.clone_str_opt(&self.current),
            select: pool.clone_str_opt(&self.select),
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
        let cmd = UpdateLabels::new(&pool, "Language", "Current", "Select", "OK", "Cancel");
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Language");
        assert_eq!(back.select.unwrap().as_str().unwrap(), "Select");
        assert_eq!(back.cancel_button.unwrap().as_str().unwrap(), "Cancel");
    }
}
