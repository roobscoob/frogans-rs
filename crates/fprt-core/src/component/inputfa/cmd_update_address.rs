//! `update_address` command (engine → host) — canonical address (pooled string).

use fprt_sys::ui::inputfa::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::inputfa::update_address::UpdateAddress as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The canonical Frogans Address text the engine wants shown in the input field
/// (e.g. after normalization).
#[derive(Debug)]
pub struct UpdateAddress {
    /// Frogans address text, or `None` if empty.
    pub address: Option<PooledString>,
}

impl UpdateAddress {
    /// Build one, allocating `address` into `pool`.
    pub fn new(pool: &OwnedPool, address: &str) -> Self {
        UpdateAddress {
            address: Some(pool.alloc_str(address)),
        }
    }

    /// Decode the engine's payload, wrapping its pooled bytes zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `address` was written into `pool` by the pop that produced both.
        UpdateAddress {
            address: unsafe { pool.string(raw.address) },
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_ADDRESS,
            address: ustring_opt(self.address.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateAddress {
            address: pool.clone_str_opt(&self.address),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateAddress::new(&pool, "frogans*réseau");
        let back = UpdateAddress::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.address.unwrap().as_str().unwrap(), "frogans*réseau");
    }
}
