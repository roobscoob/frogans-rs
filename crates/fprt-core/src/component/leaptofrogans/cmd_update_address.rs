//! `update_address` command (engine → host) — the candidate address (a single
//! **pooled** string) plus a compliance flag.

use fprt_sys::ui::leaptofrogans::CMD_UPDATE_ADDRESS;
use fprt_sys::ui::leaptofrogans::update_address::UpdateAddress as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The Frogans address being evaluated, plus whether it's compliant (drives
/// which buttons the host shows).
#[derive(Debug)]
pub struct UpdateAddress {
    /// The candidate Frogans address.
    pub address: Option<PooledString>,
    /// Whether the address is compliant.
    pub compliant: bool,
}

impl UpdateAddress {
    /// Build one, allocating `address` into `pool`.
    pub fn new(pool: &OwnedPool, address: &str, compliant: bool) -> Self {
        UpdateAddress {
            address: Some(pool.alloc_str(address)),
            compliant,
        }
    }

    /// Decode the engine's payload, wrapping its pooled bytes zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.address` was written into `pool` by the pop that produced both.
        let address = unsafe { pool.string(raw.address) };
        UpdateAddress {
            address,
            compliant: raw.compliant_address == 1,
        }
    }

    /// Encode into the raw payload (descriptor points at the bytes we hold).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_ADDRESS,
            _rsv04: 0,
            address: ustring_opt(self.address.as_ref()),
            compliant_address: self.compliant as u32,
            _rsv1c: 0,
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateAddress {
            address: pool.clone_str_opt(&self.address),
            compliant: self.compliant,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateAddress::new(&pool, "frogans*example", true);
        let back = UpdateAddress::from_raw(cmd.to_raw(), &pool.as_pool());
        assert_eq!(back.address.unwrap().as_str().unwrap(), "frogans*example");
        assert!(back.compliant);
    }

    #[test]
    fn non_compliant_roundtrips() {
        let pool = OwnedPool::new();
        let cmd = UpdateAddress::new(&pool, "not-an-address", false);
        let back = UpdateAddress::from_raw(cmd.to_raw(), &pool.as_pool());
        assert!(!back.compliant);
    }
}
