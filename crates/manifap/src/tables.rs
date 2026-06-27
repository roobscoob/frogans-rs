//! Access to the generated Unicode-7.0.0 lookup tables.
//!
//! The `static` slices come from `build.rs` (see [`crate`]-level notes); this
//! module only adds the binary-search readers that the IFAP/FACR engines call.
//! Naming mirrors the specs' pseudocode: `contains` is `TABLE.CONTAINS`, the
//! `lookup_*` helpers are `TABLE.LOOKUP(cp, #field)`.

// Data + readers; the convergence tables and a few bidi codes are wired up by
// the FACR layer as it grows, so unused-until-then items are expected here.
#![allow(dead_code)]

use core::cmp::Ordering;

include!(concat!(env!("OUT_DIR"), "/tables.rs"));

/// Bidi class codes, matching `bidi_code` in `build.rs`. `L` is the value the
/// spec substitutes whenever ILT09 returns NULL, so it is the natural `0`.
pub mod bidi {
    pub const L: u8 = 0;
    pub const R: u8 = 1;
    pub const AL: u8 = 2;
    pub const EN: u8 = 3;
    pub const ES: u8 = 4;
    pub const AN: u8 = 5;
    pub const NSM: u8 = 6;
    pub const BN: u8 = 7;
    pub const ON: u8 = 8;
}

/// `TABLE.CONTAINS(cp)` for a range/set table.
pub fn contains(set: &[(u32, u32)], cp: u32) -> bool {
    set.binary_search_by(|&(lo, hi)| range_cmp(lo, hi, cp)).is_ok()
}

/// `TABLE.LOOKUP(cp, #scalar)` for a range table carrying one scalar column.
pub fn lookup_scalar(set: &[(u32, u32, u32)], cp: u32) -> Option<u32> {
    set.binary_search_by(|&(lo, hi, _)| range_cmp(lo, hi, cp))
        .ok()
        .map(|i| set[i].2)
}

/// `TABLE.LOOKUP(cp, #array)` for a code-point -> code-point-array mapping.
pub fn lookup_mapping(map: &[(u32, &'static [u32])], cp: u32) -> Option<&'static [u32]> {
    map.binary_search_by_key(&cp, |&(k, _)| k)
        .ok()
        .map(|i| map[i].1)
}

/// `mapping_table.CONTAINS(cp)` — presence in a mapping table's first column.
pub fn mapping_contains(map: &[(u32, &'static [u32])], cp: u32) -> bool {
    map.binary_search_by_key(&cp, |&(k, _)| k).is_ok()
}

/// `ILT02.FIND((#full_composition_exclusion == 0) AND (#canonical_mapping == [a, b]))`
/// — the canonical-composition reverse lookup behind `c2_compose`.
pub fn compose(first: u32, second: u32) -> Option<u32> {
    ILT02_COMPOSE
        .binary_search_by(|&(a, b, _)| (a, b).cmp(&(first, second)))
        .ok()
        .map(|i| ILT02_COMPOSE[i].2)
}

/// Canonical combining class, defaulting to 0 (the spec's NULL substitute).
pub fn ccc(cp: u32) -> u32 {
    lookup_scalar(ILT04_CCC, cp).unwrap_or(0)
}

/// Bidi class, defaulting to `L` (the spec's NULL substitute).
pub fn bidi_class(cp: u32) -> u8 {
    lookup_scalar(ILT09_BIDI, cp).unwrap_or(bidi::L as u32) as u8
}

/// Joining type as its ASCII byte (`b'D'`, `b'L'`, …), defaulting to `U`.
pub fn joining_type(cp: u32) -> u8 {
    lookup_scalar(ILT07_JOINING, cp).map_or(b'U', |v| v as u8)
}

fn range_cmp(lo: u32, hi: u32, cp: u32) -> Ordering {
    if cp < lo {
        Ordering::Greater
    } else if cp > hi {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}
