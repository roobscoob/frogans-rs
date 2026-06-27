//! Unicode normalization over the bundled tables, transcribed from IFAP
//! Appendix C.2 (`c2_*`) and C.6 (`c6_*`).
//!
//! Everything works on `Vec<u32>` code points. Because the tables already encode
//! Hangul and every other decomposition explicitly (ILT02/ILT03), the recursive
//! decomposition here needs no special-case algorithm — it is a literal reading
//! of the pseudocode.

use crate::tables;

/// NFKC: compatibility-decompose, canonical-reorder, canonically compose.
/// (`c2_normalize_nfkc`)
pub fn nfkc(cps: &[u32]) -> Vec<u32> {
    let mut work = decompose_compatibility(cps);
    reorder(&mut work);
    compose(&mut work);
    work
}

/// NFD: canonically decompose, then reorder. (`c6_normalize_nfd`)
pub fn nfd(cps: &[u32]) -> Vec<u32> {
    let mut work = decompose_canonical(cps);
    reorder(&mut work);
    work
}

/// NFC: canonically decompose, reorder, then compose. (`c6_normalize_nfc`)
pub fn nfc(cps: &[u32]) -> Vec<u32> {
    let mut work = decompose_canonical(cps);
    reorder(&mut work);
    compose(&mut work);
    work
}

fn decompose_compatibility(cps: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(cps.len());
    for &cp in cps {
        decompose_compatibility_cp(cp, &mut out);
    }
    out
}

fn decompose_compatibility_cp(cp: u32, out: &mut Vec<u32>) {
    // Canonical mapping is consulted first, then compatibility; a code point
    // never appears in both. (`c2_decompose_compatibility_cp`)
    if let Some(mapping) = tables::lookup_mapping(tables::ILT02_CANONICAL, cp) {
        for &c in mapping {
            decompose_compatibility_cp(c, out);
        }
    } else if let Some(mapping) = tables::lookup_mapping(tables::ILT03_COMPAT, cp) {
        for &c in mapping {
            decompose_compatibility_cp(c, out);
        }
    } else {
        out.push(cp);
    }
}

fn decompose_canonical(cps: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(cps.len());
    for &cp in cps {
        decompose_canonical_cp(cp, &mut out);
    }
    out
}

fn decompose_canonical_cp(cp: u32, out: &mut Vec<u32>) {
    // (`c6_decompose_canonical_cp`)
    if let Some(mapping) = tables::lookup_mapping(tables::ILT02_CANONICAL, cp) {
        for &c in mapping {
            decompose_canonical_cp(c, out);
        }
    } else {
        out.push(cp);
    }
}

/// Canonical ordering algorithm — a stable bubble sort that swaps an adjacent
/// pair only when the later mark has a strictly smaller, non-zero combining
/// class. (`c2_reorder`)
fn reorder(cps: &mut [u32]) {
    let mut swapped = true;
    while swapped {
        swapped = false;
        for i in 1..cps.len() {
            let prev_ccc = tables::ccc(cps[i - 1]);
            let cur_ccc = tables::ccc(cps[i]);
            if cur_ccc != 0 && prev_ccc > 0 && prev_ccc > cur_ccc {
                cps.swap(i - 1, i);
                swapped = true;
            }
        }
    }
}

/// Canonical composition algorithm. (`c2_compose`)
fn compose(cps: &mut Vec<u32>) {
    if cps.is_empty() {
        return;
    }
    let mut starter_index = 0usize;
    let mut starter_ccc = tables::ccc(cps[0]);
    let mut index = 1usize;
    while index < cps.len() {
        let starter_cp = cps[starter_index];
        let prev_ccc = tables::ccc(cps[index - 1]);
        let cur_cp = cps[index];
        let cur_ccc = tables::ccc(cur_cp);

        let composite = if starter_ccc == 0 {
            tables::compose(starter_cp, cur_cp)
        } else {
            None
        };

        if let Some(composite) = composite
            && (prev_ccc < cur_ccc || prev_ccc == 0)
        {
            cps[starter_index] = composite;
            cps.remove(index);
            // `index` and `starter_ccc` stay put: the new composite remains the
            // current starter and is still a starter (ccc 0).
        } else {
            if cur_ccc == 0 {
                starter_index = index;
                starter_ccc = 0;
            }
            index += 1;
        }
    }
}
