//! `update_error_raise` command (engine → host) — inline error (pooled string).

use fprt_sys::ui::inputfa::CMD_UPDATE_ERROR_RAISE;
use fprt_sys::ui::inputfa::update_error_raise::UpdateErrorRaise as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The localized error string the engine wants shown in the dialog's inline
/// error label (the user typed an invalid Frogans address).
#[derive(Debug)]
pub struct UpdateErrorRaise {
    /// Inline error text, or `None` if empty.
    pub error_msg: Option<PooledString>,
}

impl UpdateErrorRaise {
    /// Build one, allocating `error_msg` into `pool`.
    pub fn new(pool: &OwnedPool, error_msg: &str) -> Self {
        UpdateErrorRaise {
            error_msg: Some(pool.alloc_str(error_msg)),
        }
    }

    /// Decode the engine's payload, wrapping its pooled bytes zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `error_msg` was written into `pool` by the pop that produced both.
        UpdateErrorRaise {
            error_msg: unsafe { pool.string(raw.error_msg) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_ERROR_RAISE,
            error_msg: ustring_opt(self.error_msg.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateErrorRaise {
            error_msg: pool.clone_str_opt(&self.error_msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateErrorRaise::new(&pool, "Invalid address");
        let back = UpdateErrorRaise::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.error_msg.unwrap().as_str().unwrap(), "Invalid address");
    }
}
