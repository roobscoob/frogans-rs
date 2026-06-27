//! `update_labels` command (engine → host) — four **pooled** dialog strings.

use fprt_sys::ui::update::CMD_UPDATE_LABELS;
use fprt_sys::ui::update::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The four localized strings of the software-update dialog.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Window title.
    pub window_title: Option<PooledString>,
    /// Instruction text (the notification-selected one).
    pub instruction_text: Option<PooledString>,
    /// Download-button title.
    pub download_button_title: Option<PooledString>,
    /// Cancel-button title.
    pub cancel_button_title: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        window_title: &str,
        instruction_text: &str,
        download_button_title: &str,
        cancel_button_title: &str,
    ) -> Self {
        UpdateLabels {
            window_title: Some(pool.alloc_str(window_title)),
            instruction_text: Some(pool.alloc_str(instruction_text)),
            download_button_title: Some(pool.alloc_str(download_button_title)),
            cancel_button_title: Some(pool.alloc_str(cancel_button_title)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                window_title: pool.string(raw.window_title),
                instruction_text: pool.string(raw.instruction_text),
                download_button_title: pool.string(raw.download_button_title),
                cancel_button_title: pool.string(raw.cancel_button_title),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            window_title: ustring_opt(self.window_title.as_ref()),
            instruction_text: ustring_opt(self.instruction_text.as_ref()),
            download_button_title: ustring_opt(self.download_button_title.as_ref()),
            cancel_button_title: ustring_opt(self.cancel_button_title.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            window_title: pool.clone_str_opt(&self.window_title),
            instruction_text: pool.clone_str_opt(&self.instruction_text),
            download_button_title: pool.clone_str_opt(&self.download_button_title),
            cancel_button_title: pool.clone_str_opt(&self.cancel_button_title),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateLabels::new(&pool, "Update", "A new version is available.", "Download", "Cancel");
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.window_title.unwrap().as_str().unwrap(), "Update");
        assert_eq!(
            back.instruction_text.unwrap().as_str().unwrap(),
            "A new version is available."
        );
        assert_eq!(back.cancel_button_title.unwrap().as_str().unwrap(), "Cancel");
    }
}
