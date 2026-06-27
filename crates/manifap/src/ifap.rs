//! The IFAP 1.1 engine: validate a candidate string against the Frogans address
//! pattern and generate reference forms.
//!
//! The flow follows the specification's prerequisite chain (each step assumes the
//! previous one passed): character set (§3.1) → string formation / NFKC (§3.2) →
//! eligible characters (§3.3) → directionality (§3.4) → structure (§4) → length
//! (§6). The reference form (§5) is then built per part for identity and length.
//!
//! Function names in comments map to IFAP Appendix C (`c1_*`…`c6_*`).

use crate::error::{FrogansAddressEvaluationError as Error, Part, StructureViolation};
use crate::{normalize, tables};

const SEPARATOR: u32 = 0x2A; // ASTERISK
const ZWNJ: u32 = 0x200C; // ZERO WIDTH NON-JOINER
const ZWJ: u32 = 0x200D; // ZERO WIDTH JOINER
const VIRAMA: u32 = 9; // Canonical_Combining_Class value

/// Connector characters (IFAP §4.4).
const CONNECTORS: [u32; 4] = [0x002D, 0x00B7, 0x0F0B, 0x30FB];

/// The five signs barred from the first position of a network name (IFAP §4.2).
const NETWORK_FIRST_DISALLOWED: [u32; 5] = [0x0375, 0x05F3, 0x05F4, 0x06FD, 0x06FE];

/// A validated split of a Frogans address into its reference form and parts.
pub struct Evaluation {
    /// Reference form of the whole address: `network_ref*site_ref`.
    pub reference: String,
}

/// Validate `input` and, if it is a Frogans address, produce its reference form.
pub fn evaluate(input: &str) -> Result<Evaluation, Error> {
    let cps: Vec<u32> = input.chars().map(u32::from).collect();
    if cps.is_empty() {
        return Err(Error::Empty);
    }

    verify_character_set(&cps)?; // §3.1
    verify_string_formation(&cps)?; // §3.2
    verify_eligible_characters(&cps)?; // §3.3
    verify_directionality(&cps)?; // §3.4

    // Separator + split (§4.1).
    let star_positions: Vec<usize> = cps
        .iter()
        .enumerate()
        .filter(|&(_, &cp)| cp == SEPARATOR)
        .map(|(i, _)| i)
        .collect();
    if star_positions.len() != 1 {
        return Err(Error::SeparatorCount {
            found: star_positions.len(),
        });
    }
    let star = star_positions[0];
    if star == 0 || star == cps.len() - 1 {
        return Err(Error::SeparatorAtEdge);
    }
    let network = &cps[..star];
    let site = &cps[star + 1..];

    verify_structure(network, Part::NetworkName)?; // §4
    verify_structure(site, Part::SiteName)?;

    // Reference form + length (§5, §6).
    let network_ref = reference_form(network);
    let site_ref = reference_form(site);
    check_length(&network_ref, Part::NetworkName)?;
    check_length(&site_ref, Part::SiteName)?;

    let mut reference = String::new();
    push_cps(&mut reference, &network_ref);
    reference.push('*');
    push_cps(&mut reference, &site_ref);

    Ok(Evaluation { reference })
}

/// `c1_verify_character_set`.
fn verify_character_set(cps: &[u32]) -> Result<(), Error> {
    for (index, &cp) in cps.iter().enumerate() {
        if !tables::contains(tables::ILT01_CHARACTER_SET, cp) {
            return Err(Error::OutsideCharacterSet {
                index,
                code_point: cp,
            });
        }
    }
    Ok(())
}

/// `c2_verify_string_formation`.
fn verify_string_formation(cps: &[u32]) -> Result<(), Error> {
    if normalize::nfkc(cps) != cps {
        return Err(Error::NotNormalized);
    }

    let mut run = 0u32;
    for &cp in cps {
        if tables::contains(tables::ILT06_COMBINING_MARKS, cp) {
            run += 1;
            if run > 30 {
                return Err(Error::ExcessiveCombiningMarks);
            }
        } else {
            run = 0;
        }
    }

    for (index, &cp) in cps.iter().enumerate() {
        if cp == ZWNJ && !(verify_joiner_200c_sequence(cps, index) || verify_joiner_virama(cps, index))
        {
            return Err(Error::ContextualJoiner {
                index,
                code_point: cp,
            });
        }
        if cp == ZWJ && !verify_joiner_virama(cps, index) {
            return Err(Error::ContextualJoiner {
                index,
                code_point: cp,
            });
        }
    }
    Ok(())
}

/// `c2_verify_joiner_200c_sequence` — a join-causing letter on each side of the
/// ZWNJ, skipping transparent (combining) characters.
fn verify_joiner_200c_sequence(cps: &[u32], joiner: usize) -> bool {
    if cps.len() < 3 || joiner == 0 || joiner == cps.len() - 1 {
        return false;
    }
    let mut start_found = false;
    for i in (0..joiner).rev() {
        let jt = tables::joining_type(cps[i]);
        if jt != b'T' {
            start_found = jt == b'D' || jt == b'L';
            break;
        }
    }
    if !start_found {
        return false;
    }
    for &cp in &cps[joiner + 1..] {
        let jt = tables::joining_type(cp);
        if jt != b'T' {
            return jt == b'D' || jt == b'R';
        }
    }
    false
}

/// `c2_verify_joiner_virama` — the joiner is preceded by a Virama (ccc 9).
fn verify_joiner_virama(cps: &[u32], joiner: usize) -> bool {
    joiner != 0 && tables::ccc(cps[joiner - 1]) == VIRAMA
}

/// `c3_verify_eligible_characters`.
fn verify_eligible_characters(cps: &[u32]) -> Result<(), Error> {
    for (index, &cp) in cps.iter().enumerate() {
        if !tables::contains(tables::ILT08_ELIGIBLE, cp) {
            return Err(Error::Ineligible {
                index,
                code_point: cp,
            });
        }
    }
    Ok(())
}

/// `c4_verify_directionality` over the whole address (separator included).
fn verify_directionality(cps: &[u32]) -> Result<(), Error> {
    use tables::bidi;
    let first = tables::bidi_class(cps[0]);
    let ok = match first {
        bidi::L => verify_ltr(cps),
        bidi::R | bidi::AL => verify_rtl(cps),
        _ => false, // first code point lacks strong directionality
    };
    if ok { Ok(()) } else { Err(Error::Directionality) }
}

/// `c4_verify_ltr`.
fn verify_ltr(cps: &[u32]) -> bool {
    use tables::bidi;
    for &cp in &cps[1..] {
        let b = tables::bidi_class(cp);
        if b == bidi::R || b == bidi::AL || b == bidi::AN {
            return false;
        }
    }
    for &cp in cps[1..].iter().rev() {
        let b = tables::bidi_class(cp);
        if b != bidi::NSM {
            return b == bidi::L || b == bidi::EN;
        }
    }
    true
}

/// `c4_verify_rtl`.
fn verify_rtl(cps: &[u32]) -> bool {
    use tables::bidi;
    for &cp in &cps[1..] {
        if tables::bidi_class(cp) == bidi::L {
            return false;
        }
    }
    for &cp in cps[1..].iter().rev() {
        let b = tables::bidi_class(cp);
        if b != bidi::NSM {
            return b == bidi::R || b == bidi::AL || b == bidi::EN || b == bidi::AN;
        }
    }
    true
}

/// `c5_verify_structure_network_name` / `c5_verify_structure_site_name`.
fn verify_structure(part: &[u32], which: Part) -> Result<(), Error> {
    let fail = |violation| {
        Err(Error::Structure {
            part: which,
            violation,
        })
    };

    if part.contains(&SEPARATOR) {
        return fail(StructureViolation::ContainsSeparator);
    }

    let first = part[0];
    if tables::contains(tables::ILT06_COMBINING_MARKS, first) {
        return fail(StructureViolation::FirstCharacterCombiningMark);
    }
    if which == Part::NetworkName {
        if tables::contains(tables::ILT10_DECIMAL, first) {
            return fail(StructureViolation::FirstCharacterDecimalNumber);
        }
        if NETWORK_FIRST_DISALLOWED.contains(&first) {
            return fail(StructureViolation::FirstCharacterDisallowed);
        }
    }

    verify_connectors(part, which)
}

/// `c5_verify_connector_characters`.
fn verify_connectors(part: &[u32], which: Part) -> Result<(), Error> {
    let fail = |violation| {
        Err(Error::Structure {
            part: which,
            violation,
        })
    };
    let is_connector = |cp: u32| CONNECTORS.contains(&cp);

    if is_connector(part[0]) || is_connector(part[part.len() - 1]) {
        return fail(StructureViolation::ConnectorAtEdge);
    }
    for window in part.windows(2) {
        let (prev, cur) = (window[0], window[1]);
        if is_connector(prev) {
            if is_connector(cur) {
                return fail(StructureViolation::ConsecutiveConnectors);
            }
            if tables::contains(tables::ILT06_COMBINING_MARKS, cur) {
                return fail(StructureViolation::ConnectorFollowedByCombiningMark);
            }
        }
    }
    Ok(())
}

/// `c6_generate_reference_form`: NFD, then NFKC case fold, then NFC.
pub fn reference_form(part: &[u32]) -> Vec<u32> {
    let nfd = normalize::nfd(part);
    let mut folded = Vec::with_capacity(nfd.len());
    for cp in nfd {
        match tables::lookup_mapping(tables::ILT11_FOLD, cp) {
            Some(mapping) => folded.extend_from_slice(mapping),
            None => folded.push(cp),
        }
    }
    normalize::nfc(&folded)
}

fn check_length(reference: &[u32], part: Part) -> Result<(), Error> {
    let n = reference.len();
    if (1..=28).contains(&n) {
        Ok(())
    } else {
        Err(Error::Length {
            part,
            characters: n,
        })
    }
}

fn push_cps(s: &mut String, cps: &[u32]) {
    for &cp in cps {
        s.push(char::from_u32(cp).expect("reference-form code points are valid scalars"));
    }
}
