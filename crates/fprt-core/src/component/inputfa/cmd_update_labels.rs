//! `update_labels` command (engine → host) — five **pooled** dialog strings.

use fprt_sys::ui::inputfa::CMD_UPDATE_LABELS;
use fprt_sys::ui::inputfa::labels::Labels as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The five localized strings labelling the input-FA dialog chrome.
#[derive(Debug)]
pub struct UpdateLabels {
    /// Window title.
    pub window_title: Option<PooledString>,
    /// Instruction text.
    pub instruction: Option<PooledString>,
    /// Input-field placeholder.
    pub input_placeholder: Option<PooledString>,
    /// OK-button label.
    pub ok_button_title: Option<PooledString>,
    /// Close-button label.
    pub close_button_title: Option<PooledString>,
}

impl UpdateLabels {
    /// Build one, allocating each label into `pool`.
    pub fn new(
        pool: &OwnedPool,
        window_title: &str,
        instruction: &str,
        input_placeholder: &str,
        ok_button_title: &str,
        close_button_title: &str,
    ) -> Self {
        UpdateLabels {
            window_title: Some(pool.alloc_str(window_title)),
            instruction: Some(pool.alloc_str(instruction)),
            input_placeholder: Some(pool.alloc_str(input_placeholder)),
            ok_button_title: Some(pool.alloc_str(ok_button_title)),
            close_button_title: Some(pool.alloc_str(close_button_title)),
        }
    }

    /// Decode the engine's payload, wrapping each pooled string zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every field was written into `pool` by the pop that produced both.
        unsafe {
            UpdateLabels {
                window_title: pool.string(raw.window_title),
                instruction: pool.string(raw.instruction),
                input_placeholder: pool.string(raw.input_placeholder),
                ok_button_title: pool.string(raw.ok_button_title),
                close_button_title: pool.string(raw.close_button_title),
            }
        }
    }

    /// Encode into the raw payload (descriptors point at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_LABELS,
            window_title: ustring_opt(self.window_title.as_ref()),
            instruction: ustring_opt(self.instruction.as_ref()),
            input_placeholder: ustring_opt(self.input_placeholder.as_ref()),
            ok_button_title: ustring_opt(self.ok_button_title.as_ref()),
            close_button_title: ustring_opt(self.close_button_title.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateLabels {
            window_title: pool.clone_str_opt(&self.window_title),
            instruction: pool.clone_str_opt(&self.instruction),
            input_placeholder: pool.clone_str_opt(&self.input_placeholder),
            ok_button_title: pool.clone_str_opt(&self.ok_button_title),
            close_button_title: pool.clone_str_opt(&self.close_button_title),
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
            &pool,
            "Input Address",
            "Type a Frogans address",
            "frogans*…",
            "OK",
            "Close",
        );
        let back = UpdateLabels::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.window_title.unwrap().as_str().unwrap(), "Input Address");
        assert_eq!(back.input_placeholder.unwrap().as_str().unwrap(), "frogans*…");
        assert_eq!(back.close_button_title.unwrap().as_str().unwrap(), "Close");
    }
}
