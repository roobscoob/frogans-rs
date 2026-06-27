//! `update_addresses` command (engine → host) — a **pooled** address list.
//!
//! The list codec: `from_raw` reads `count` descriptors out of the pool;
//! `to_raw` allocates a fresh descriptor array *into* the pool (hence the
//! `&OwnedPool`), each pointing at a string the pool already owns.

use fprt_sys::ui::AddressList as Raw;
use fprt_sys::ui::favorites::CMD_UPDATE_ADDRESSES;
use fprt_sys::ustring::Ustring;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// The list of favorite Frogans addresses the dialog shows.
#[derive(Debug)]
pub struct UpdateAddresses {
    /// The addresses (null/empty entries dropped on decode).
    pub addresses: Vec<PooledString>,
}

impl UpdateAddresses {
    /// Build one, allocating each address into `pool`.
    pub fn new(pool: &OwnedPool, addresses: &[&str]) -> Self {
        UpdateAddresses {
            addresses: addresses.iter().map(|s| pool.alloc_str(s)).collect(),
        }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.items` is `raw.count` `Ustring`s the pop wrote into `pool`.
        UpdateAddresses {
            addresses: unsafe { pool.strings(raw.items, raw.count) },
        }
    }

    /// Encode into the raw payload, allocating the descriptor array into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        let (count, items) = if self.addresses.is_empty() {
            (0, core::ptr::null())
        } else {
            let descriptors: Vec<Ustring> =
                self.addresses.iter().map(|s| ustring_opt(Some(s))).collect();
            (
                self.addresses.len() as u32,
                pool.alloc_slice(&descriptors).cast::<Ustring>(),
            )
        };
        Raw {
            status_id: CMD_UPDATE_ADDRESSES,
            _rsv04: 0,
            count,
            items,
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateAddresses {
            addresses: self.addresses.iter().map(|s| pool.clone_str(s)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateAddresses::new(&pool, &["frogans*alpha", "frogans*beta", "frogans*gamma"]);
        let back = UpdateAddresses::from_raw(cmd.to_raw(&pool), &pool.as_pool());
        let got: Vec<&str> = back.addresses.iter().map(|s| s.as_str().unwrap()).collect();
        assert_eq!(got, ["frogans*alpha", "frogans*beta", "frogans*gamma"]);
    }

    #[test]
    fn empty_list_encodes_null() {
        let pool = OwnedPool::new();
        let raw = UpdateAddresses { addresses: vec![] }.to_raw(&pool);
        assert!(raw.items.is_null());
        assert_eq!(raw.count, 0);
    }
}
