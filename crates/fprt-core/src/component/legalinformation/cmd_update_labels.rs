//! `update_labels` command (engine → host) — title + the button strings
//! (five **pooled** strings).

use fprt_sys::ui::legalinformation::CMD_UPDATE_LABELS;
use fprt_sys::ui::legalinformation::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The five localized strings of the legal-information panel.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Panel title.
    pub title: Option<PooledString>,
    /// Open-button label.
    pub open_button: Option<PooledString>,
    /// Select-button label.
    pub select_button: Option<PooledString>,
    /// Back-button label.
    pub back_button: Option<PooledString>,
    /// Close-button label.
    pub close_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        open_button: &str,
        select_button: &str,
        back_button: &str,
        close_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            open_button: Some(pool.alloc_str(open_button)),
            select_button: Some(pool.alloc_str(select_button)),
            back_button: Some(pool.alloc_str(back_button)),
            close_button: Some(pool.alloc_str(close_button)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                open_button: pool.string(raw.open_button),
                select_button: pool.string(raw.select_button),
                back_button: pool.string(raw.back_button),
                close_button: pool.string(raw.close_button),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            variant_index: 0,
            title: ustring_opt(self.title.as_ref()),
            open_button: ustring_opt(self.open_button.as_ref()),
            select_button: ustring_opt(self.select_button.as_ref()),
            back_button: ustring_opt(self.back_button.as_ref()),
            close_button: ustring_opt(self.close_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            open_button: pool.clone_str_opt(&self.open_button),
            select_button: pool.clone_str_opt(&self.select_button),
            back_button: pool.clone_str_opt(&self.back_button),
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
        let cmd = UpdateLabels::new(&pool, "Legal information", "Open", "Select", "Back", "Close");
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Legal information");
        assert_eq!(back.open_button.unwrap().as_str().unwrap(), "Open");
        assert_eq!(back.close_button.unwrap().as_str().unwrap(), "Close");
    }
}
