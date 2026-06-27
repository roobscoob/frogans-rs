//! `update_address` command (engine → host) — one **pooled** address string.

use fprt_sys::ui::inspector::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::inspector::update_address::UpdateAddress as Raw;

use crate::component::inspector::InspectorId;
use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The Frogans address text shown in one inspector window's address field.
#[derive(Debug)]
pub struct UpdateAddress {
    /// The target window.
    pub id: InspectorId,
    /// Frogans address text.
    pub address: Option<PooledString>,
}

impl UpdateAddress {
    /// Build one, allocating `address` into `pool`.
    pub fn new(pool: &OwnedPool, id: InspectorId, address: &str) -> Self {
        UpdateAddress {
            id,
            address: Some(pool.alloc_str(address)),
        }
    }

    /// Decode the engine's payload, wrapping its pooled bytes zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `address` was written into `pool` by the pop that produced both.
        UpdateAddress {
            id: InspectorId(raw.reference),
            address: unsafe { pool.string(raw.address) },
        }
    }

    /// Encode into the raw payload (descriptor points at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_ADDRESS,
            reference: self.id.0,
            address: ustring_opt(self.address.as_ref()),
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateAddress {
            id: self.id,
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
        let cmd = UpdateAddress::new(&pool, InspectorId(3), "frogans*example");
        let back = UpdateAddress::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.id, InspectorId(3));
        assert_eq!(back.address.unwrap().as_str().unwrap(), "frogans*example");
    }
}
