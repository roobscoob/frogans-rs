//! `update_labels` command (engine → host) — title + the button strings
//! (seven **pooled** dialog strings).

use fprt_sys::ui::leaptofrogans::CMD_UPDATE_LABELS;
use fprt_sys::ui::leaptofrogans::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The leap-to-Frogans dialog's strings. Two-armed: the normal view fills
/// `confirm`/`cancel`/`block`; the error view fills `close` and puts the error
/// text in `instruction`. The host detects the arm by whether `close_button` is
/// present.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Dialog title.
    pub title: Option<PooledString>,
    /// Instruction (or, in the error arm, the error text).
    pub instruction: Option<PooledString>,
    /// Confirm-button label.
    pub confirm_button: Option<PooledString>,
    /// Cancel-button label.
    pub cancel_button: Option<PooledString>,
    /// Block-button label.
    pub block_button: Option<PooledString>,
    /// Close-button label (present in the error arm).
    pub close_button: Option<PooledString>,
    /// Purge-button label (no macOS widget).
    pub purge_button: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: &OwnedPool,
        title: &str,
        instruction: &str,
        confirm_button: &str,
        cancel_button: &str,
        block_button: &str,
        close_button: &str,
        purge_button: &str,
    ) -> Self {
        UpdateLabels {
            title: Some(pool.alloc_str(title)),
            instruction: Some(pool.alloc_str(instruction)),
            confirm_button: Some(pool.alloc_str(confirm_button)),
            cancel_button: Some(pool.alloc_str(cancel_button)),
            block_button: Some(pool.alloc_str(block_button)),
            close_button: Some(pool.alloc_str(close_button)),
            purge_button: Some(pool.alloc_str(purge_button)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                title: pool.string(raw.title),
                instruction: pool.string(raw.instruction),
                confirm_button: pool.string(raw.confirm_button),
                cancel_button: pool.string(raw.cancel_button),
                block_button: pool.string(raw.block_button),
                close_button: pool.string(raw.close_button),
                purge_button: pool.string(raw.purge_button),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            selector: 0,
            title: ustring_opt(self.title.as_ref()),
            instruction: ustring_opt(self.instruction.as_ref()),
            confirm_button: ustring_opt(self.confirm_button.as_ref()),
            cancel_button: ustring_opt(self.cancel_button.as_ref()),
            block_button: ustring_opt(self.block_button.as_ref()),
            close_button: ustring_opt(self.close_button.as_ref()),
            purge_button: ustring_opt(self.purge_button.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            title: pool.clone_str_opt(&self.title),
            instruction: pool.clone_str_opt(&self.instruction),
            confirm_button: pool.clone_str_opt(&self.confirm_button),
            cancel_button: pool.clone_str_opt(&self.cancel_button),
            block_button: pool.clone_str_opt(&self.block_button),
            close_button: pool.clone_str_opt(&self.close_button),
            purge_button: pool.clone_str_opt(&self.purge_button),
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
            &pool, "Leap", "Open this site?", "Confirm", "Cancel", "Block", "Close", "Purge",
        );
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.title.unwrap().as_str().unwrap(), "Leap");
        assert_eq!(back.instruction.unwrap().as_str().unwrap(), "Open this site?");
        assert_eq!(back.purge_button.unwrap().as_str().unwrap(), "Purge");
    }
}
