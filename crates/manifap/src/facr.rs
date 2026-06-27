//! The FACR 1.0 layer: composition rules the registry applies on top of IFAP.
//!
//! Unlike IFAP, FACR is meaningful only *within a linguistic category* (declared
//! when a Frogans network is registered, and not always inferable from the
//! string because categories can overlap). So this module is keyed on
//! [`LinguisticCategory`] rather than hanging off [`FrogansAddress`].
//!
//! [`FrogansAddress`]: crate::FrogansAddress
//!
//! Implemented:
//! * employable characters (§4.1) and arrangement rules (§4.2), exposed as the
//!   per-category validity checks [`FrogansAddress::validate_network_name`] /
//!   [`validate_site_name`](FrogansAddress::validate_site_name);
//! * convergence-form generation (§7 / Appendix C.3–C.4);
//! * the FACR 1.1 site-name convergence test ([`site_names_converge`], §9) and
//!   the full FACR 1.0 network-name convergence method, overlapping categories
//!   included ([`network_names_converge`], §8).

use core::fmt;

use crate::{FrogansAddress, normalize, tables};

/// A FACR linguistic category — a language or group of languages sharing a
/// writing system, declared for a Frogans network at registration (§3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LinguisticCategory {
    /// `LC-Latin`.
    Latin,
    /// `LC-Chinese`.
    Chinese,
    /// `LC-Japanese`.
    Japanese,
    /// `LC-Korean`.
    Korean,
    /// `LC-Arabic`.
    Arabic,
    /// `LC-Cyrillic`.
    Cyrillic,
    /// `LC-Hebrew`.
    Hebrew,
    /// `LC-Devanagari`.
    Devanagari,
    /// `LC-Thai`.
    Thai,
    /// `LC-Greek`.
    Greek,
}

impl LinguisticCategory {
    /// Every category defined in FACR 1.0, in specification order.
    pub const ALL: [LinguisticCategory; 10] = [
        LinguisticCategory::Latin,
        LinguisticCategory::Chinese,
        LinguisticCategory::Japanese,
        LinguisticCategory::Korean,
        LinguisticCategory::Arabic,
        LinguisticCategory::Cyrillic,
        LinguisticCategory::Hebrew,
        LinguisticCategory::Devanagari,
        LinguisticCategory::Thai,
        LinguisticCategory::Greek,
    ];

    /// The category's canonical `LC-…` label.
    pub fn label(self) -> &'static str {
        match self {
            LinguisticCategory::Latin => "LC-Latin",
            LinguisticCategory::Chinese => "LC-Chinese",
            LinguisticCategory::Japanese => "LC-Japanese",
            LinguisticCategory::Korean => "LC-Korean",
            LinguisticCategory::Arabic => "LC-Arabic",
            LinguisticCategory::Cyrillic => "LC-Cyrillic",
            LinguisticCategory::Hebrew => "LC-Hebrew",
            LinguisticCategory::Devanagari => "LC-Devanagari",
            LinguisticCategory::Thai => "LC-Thai",
            LinguisticCategory::Greek => "LC-Greek",
        }
    }

    /// Parse an `LC-…` label, e.g. `"LC-Latin"`.
    pub fn from_label(label: &str) -> Option<LinguisticCategory> {
        LinguisticCategory::ALL
            .into_iter()
            .find(|lc| lc.label() == label)
    }

    /// The employable-character set for this category (FLT01–FLT10).
    fn employable_table(self) -> &'static [(u32, u32)] {
        match self {
            LinguisticCategory::Latin => tables::FLT01_LATIN_EMPLOYABLE,
            LinguisticCategory::Chinese => tables::FLT02_CHINESE_EMPLOYABLE,
            LinguisticCategory::Japanese => tables::FLT03_JAPANESE_EMPLOYABLE,
            LinguisticCategory::Korean => tables::FLT04_KOREAN_EMPLOYABLE,
            LinguisticCategory::Arabic => tables::FLT05_ARABIC_EMPLOYABLE,
            LinguisticCategory::Cyrillic => tables::FLT06_CYRILLIC_EMPLOYABLE,
            LinguisticCategory::Hebrew => tables::FLT07_HEBREW_EMPLOYABLE,
            LinguisticCategory::Devanagari => tables::FLT08_DEVANAGARI_EMPLOYABLE,
            LinguisticCategory::Thai => tables::FLT09_THAI_EMPLOYABLE,
            LinguisticCategory::Greek => tables::FLT10_GREEK_EMPLOYABLE,
        }
    }

    /// Whether `cp` is an employable character of this category (§4.1).
    pub fn is_employable(self, cp: u32) -> bool {
        tables::contains(self.employable_table(), cp)
    }

    /// Whether every character of `name` is employable in this category
    /// (`c1_verify_employable_characters`). This is the employable-character
    /// half of FACR validity; arrangement rules (§4.2) are applied separately.
    pub fn employable_characters_ok(self, name: &str) -> bool {
        name.chars().all(|c| self.is_employable(u32::from(c)))
    }

    /// Generate every Intra-LC convergence form of `name` for this category
    /// (`c3_generate_intra_lc_convergence_form`), one per type. Most categories
    /// have one ("…-Confusable"); Chinese additionally has a "…-Variant" type
    /// (Simplified/Traditional). `name` is the **preferred form** of a network or
    /// site name — not the reference form, and not a whole `network*site` address.
    pub fn intra_lc_convergence_forms(self, name: &str) -> Vec<ConvergenceForm> {
        let form = |label, table| ConvergenceForm {
            type_label: label,
            value: skeleton(name, table),
        };
        match self {
            LinguisticCategory::Latin => {
                vec![form("Intra-LC-Latin-Confusable", tables::FLT12_LATIN_CONFUSABLE)]
            }
            LinguisticCategory::Chinese => vec![
                form("Intra-LC-Chinese-Confusable", tables::FLT13_CHINESE_CONFUSABLE),
                form("Intra-LC-Chinese-Variant", tables::FLT14_CHINESE_VARIANT),
            ],
            LinguisticCategory::Japanese => {
                vec![form("Intra-LC-Japanese-Confusable", tables::FLT15_JAPANESE_CONFUSABLE)]
            }
            LinguisticCategory::Korean => {
                vec![form("Intra-LC-Korean-Confusable", tables::FLT16_KOREAN_CONFUSABLE)]
            }
            LinguisticCategory::Arabic => {
                vec![form("Intra-LC-Arabic-Confusable", tables::FLT17_ARABIC_CONFUSABLE)]
            }
            LinguisticCategory::Cyrillic => {
                vec![form("Intra-LC-Cyrillic-Confusable", tables::FLT18_CYRILLIC_CONFUSABLE)]
            }
            LinguisticCategory::Hebrew => {
                vec![form("Intra-LC-Hebrew-Confusable", tables::FLT19_HEBREW_CONFUSABLE)]
            }
            LinguisticCategory::Devanagari => {
                vec![form("Intra-LC-Devanagari-Confusable", tables::FLT20_DEVANAGARI_CONFUSABLE)]
            }
            LinguisticCategory::Thai => {
                vec![form("Intra-LC-Thai-Confusable", tables::FLT21_THAI_CONFUSABLE)]
            }
            LinguisticCategory::Greek => {
                vec![form("Intra-LC-Greek-Confusable", tables::FLT22_GREEK_CONFUSABLE)]
            }
        }
    }
}

/// The single Inter-LC convergence form of a network `name`
/// (`c4_generate_inter_lc_convergence_form`). It is category-independent and
/// applies to **network names only**, never site names (§7.2).
pub fn inter_lc_convergence_form(name: &str) -> ConvergenceForm {
    ConvergenceForm {
        type_label: "Inter-LC",
        value: skeleton(name, tables::FLT23_INTER_LC),
    }
}

/// A convergence form: a confusable/variant skeleton of a name under one
/// convergence-form *type* (§3.2). Two names "converge" when they share a form
/// of the *same type* — so equality here pairs the type label with the skeleton
/// value, making `ConvergenceForm` a lawful, hashable bucket key (the way the
/// registry stores them). It is **not** a Frogans address and is not for display.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConvergenceForm {
    type_label: &'static str,
    value: String,
}

impl ConvergenceForm {
    /// The convergence-form type label, e.g. `"Intra-LC-Latin-Confusable"`.
    pub fn type_label(&self) -> &'static str {
        self.type_label
    }

    /// The skeleton string itself.
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// `c3_generate_convergence_form`: NFD, map each code point through the skeleton
/// table (1-to-n, or itself when absent), then NFD again. Operates on the
/// preferred-form code points of a single name.
fn skeleton(name: &str, table: &[(u32, &'static [u32])]) -> String {
    let cps: Vec<u32> = name.chars().map(u32::from).collect();
    let decomposed = normalize::nfd(&cps);

    let mut mapped = Vec::with_capacity(decomposed.len());
    for cp in decomposed {
        match tables::lookup_mapping(table, cp) {
            Some(to) => mapped.extend_from_slice(to),
            None => mapped.push(cp),
        }
    }

    normalize::nfd(&mapped)
        .into_iter()
        .map(|cp| char::from_u32(cp).expect("skeleton code points are valid scalars"))
        .collect()
}

impl FrogansAddress {
    /// The convergence forms of this address's **network name** under `lc`: the
    /// category's Intra-LC form(s) plus the one Inter-LC form (§7).
    pub fn network_convergence_forms(&self, lc: LinguisticCategory) -> Vec<ConvergenceForm> {
        let name = self.network_name();
        let mut forms = lc.intra_lc_convergence_forms(name);
        forms.push(inter_lc_convergence_form(name));
        forms
    }

    /// The convergence forms of this address's **site name** under `lc`: the
    /// category's Intra-LC form(s) only — site names have no Inter-LC form (§7.2).
    pub fn site_convergence_forms(&self, lc: LinguisticCategory) -> Vec<ConvergenceForm> {
        lc.intra_lc_convergence_forms(self.site_name())
    }
}

/// Whether two site names are convergent, per the **FACR 1.1** method (§9 as
/// replaced by FACR 1.1 §3 — the simplified test that no longer considers
/// overlapping linguistic categories).
///
/// The two addresses are assumed to share a common network name and the given
/// linguistic category `lc` (site names are only comparable within one network).
/// The check is: the site names are IFAP-identical, or they share an Intra-LC
/// convergence form of some type.
pub fn site_names_converge(lc: LinguisticCategory, a: &FrogansAddress, b: &FrogansAddress) -> bool {
    // A. Identical site names according to IFAP (compare reference-form halves).
    if site_reference(a) == site_reference(b) {
        return true;
    }
    // B.1. A shared Intra-LC convergence form of any type.
    let a_forms = a.site_convergence_forms(lc);
    let b_forms = b.site_convergence_forms(lc);
    a_forms.iter().any(|form| b_forms.contains(form))
}

/// The reference-form site name (the part after the separator in the reference
/// form). Exactly one separator is present in any validated address.
fn site_reference(address: &FrogansAddress) -> &str {
    address
        .reference_form()
        .split_once('*')
        .expect("a validated address has exactly one separator")
        .1
}

/// The reference-form network name (the part before the separator).
fn network_reference(address: &FrogansAddress) -> &str {
    address
        .reference_form()
        .split_once('*')
        .expect("a validated address has exactly one separator")
        .0
}

// ---- FACR validity (§5): employable characters + arrangement rules ----------

/// Connector characters (IFAP §4.4), shared by several arrangement rules.
const CONNECTORS: [u32; 4] = [0x002D, 0x00B7, 0x0F0B, 0x30FB];

/// Why a network/site name is not valid for a linguistic category (FACR §4–§5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FacrError {
    /// A character is not employable in the category (§4.1).
    NotEmployable {
        /// Code-point index within the name.
        index: usize,
        /// The offending code point.
        code_point: u32,
    },
    /// The name mixes two different connector characters (§4.2).
    MixedConnectors,
    /// A U+00B7 MIDDLE DOT is not in the required `l·l` / `L·L` context (§10.1.3).
    MiddleDotContext,
    /// A U+30FB KATAKANA MIDDLE DOT is not flanked by Han/Kana characters (§10.3.3).
    KatakanaMiddleDotContext,
    /// The name lacks any character native to the category's writing system (§6).
    MissingNativeCharacter,
    /// The name mixes decimal digits from different numbering systems (§4.2).
    MixedDecimalSystems,
    /// A site name uses a different connector character than its network name (§4.2).
    ConnectorMismatchWithNetwork,
    /// A site name uses a different decimal system than its network name (§4.2).
    DecimalSystemMismatchWithNetwork,
}

impl fmt::Display for FacrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FacrError as E;
        match self {
            E::NotEmployable { index, code_point } => write!(
                f,
                "U+{code_point:04X} at position {index} is not employable in this category"
            ),
            E::MixedConnectors => write!(f, "the name mixes different connector characters"),
            E::MiddleDotContext => {
                write!(f, "a middle dot (U+00B7) is not between two 'l'/'L' characters")
            }
            E::KatakanaMiddleDotContext => {
                write!(f, "a katakana middle dot (U+30FB) is not flanked by Han/Kana characters")
            }
            E::MissingNativeCharacter => {
                write!(f, "the name has no character native to this category's writing system")
            }
            E::MixedDecimalSystems => {
                write!(f, "the name mixes decimal digits from different numbering systems")
            }
            E::ConnectorMismatchWithNetwork => {
                write!(f, "the site name uses a different connector than its network name")
            }
            E::DecimalSystemMismatchWithNetwork => {
                write!(f, "the site name uses a different decimal system than its network name")
            }
        }
    }
}

impl std::error::Error for FacrError {}

impl FrogansAddress {
    /// Validate this address's **network name** for `lc`: employable characters
    /// (§4.1) plus the category's arrangement rules (§4.2). On success the
    /// network name is a valid network name for `lc` (FACR §5).
    pub fn validate_network_name(&self, lc: LinguisticCategory) -> Result<(), FacrError> {
        let cps: Vec<u32> = self.network_name().chars().map(u32::from).collect();
        verify_employable(lc, &cps)?;
        verify_network_arrangement(lc, &cps)
    }

    /// Validate this address's **site name** for `lc`, in the context of its own
    /// network name. Applies employable characters (§4.1) and the category's
    /// site arrangement rules (§4.2): the network's rules minus the native-
    /// character rule, plus the connector- and decimal-system coupling with the
    /// network name.
    pub fn validate_site_name(&self, lc: LinguisticCategory) -> Result<(), FacrError> {
        let network: Vec<u32> = self.network_name().chars().map(u32::from).collect();
        let site: Vec<u32> = self.site_name().chars().map(u32::from).collect();
        verify_employable(lc, &site)?;
        verify_site_arrangement(lc, &network, &site)
    }

    /// Whether the network name is valid for `lc` (the boolean form of
    /// [`validate_network_name`](FrogansAddress::validate_network_name)).
    pub fn is_valid_network_name(&self, lc: LinguisticCategory) -> bool {
        self.validate_network_name(lc).is_ok()
    }

    /// Whether the site name is valid for `lc` (the boolean form of
    /// [`validate_site_name`](FrogansAddress::validate_site_name)).
    pub fn is_valid_site_name(&self, lc: LinguisticCategory) -> bool {
        self.validate_site_name(lc).is_ok()
    }
}

fn verify_employable(lc: LinguisticCategory, cps: &[u32]) -> Result<(), FacrError> {
    for (index, &cp) in cps.iter().enumerate() {
        if !lc.is_employable(cp) {
            return Err(FacrError::NotEmployable {
                index,
                code_point: cp,
            });
        }
    }
    Ok(())
}

/// `c2_verify_arrangement_rules` for a network name.
fn verify_network_arrangement(lc: LinguisticCategory, cps: &[u32]) -> Result<(), FacrError> {
    use LinguisticCategory as L;
    match lc {
        L::Latin => {
            rule_single_connector(cps)?;
            rule_middle_dot(cps)?;
        }
        L::Chinese => rule_native(cps, tables::FLT02_CHINESE_NATIVE)?,
        L::Japanese => {
            rule_single_connector(cps)?;
            rule_native(cps, tables::FLT03_JAPANESE_NATIVE)?;
            rule_katakana_middle_dot(cps)?;
        }
        L::Korean => rule_native(cps, tables::FLT04_KOREAN_NATIVE)?,
        L::Arabic | L::Devanagari => rule_single_decimal_system(cps)?,
        L::Thai => {
            rule_native(cps, tables::FLT09_THAI_NATIVE)?;
            rule_single_decimal_system(cps)?;
        }
        L::Cyrillic | L::Hebrew | L::Greek => {}
    }
    Ok(())
}

/// `c2_verify_arrangement_rules` adapted for a site name: the network rules
/// without the native-character rule, plus coupling to the network name.
fn verify_site_arrangement(
    lc: LinguisticCategory,
    network: &[u32],
    site: &[u32],
) -> Result<(), FacrError> {
    use LinguisticCategory as L;
    match lc {
        L::Latin => {
            rule_single_connector(site)?;
            rule_middle_dot(site)?;
        }
        L::Japanese => {
            rule_single_connector(site)?;
            rule_katakana_middle_dot(site)?;
        }
        L::Arabic | L::Devanagari | L::Thai => rule_single_decimal_system(site)?,
        L::Chinese | L::Korean | L::Cyrillic | L::Hebrew | L::Greek => {}
    }
    rule_connector_matches_network(network, site)?;
    rule_decimal_matches_network(network, site)?;
    Ok(())
}

/// `c2_verify_arrangement_rule_connectors`: a name may not mix two different
/// connector characters.
fn rule_single_connector(cps: &[u32]) -> Result<(), FacrError> {
    let mut first = None;
    for &cp in cps {
        if CONNECTORS.contains(&cp) {
            match first {
                None => first = Some(cp),
                Some(f) if f != cp => return Err(FacrError::MixedConnectors),
                Some(_) => {}
            }
        }
    }
    Ok(())
}

/// `c2_verify_arrangement_rule_middle_dot`: U+00B7 only between two `l`/`L`.
fn rule_middle_dot(cps: &[u32]) -> Result<(), FacrError> {
    for i in 0..cps.len() {
        if cps[i] == 0x00B7 {
            if i == 0 || i == cps.len() - 1 {
                return Err(FacrError::MiddleDotContext);
            }
            let prev = cps[i - 1];
            let next = cps[i + 1];
            if (prev != 0x006C && prev != 0x004C) || next != prev {
                return Err(FacrError::MiddleDotContext);
            }
        }
    }
    Ok(())
}

/// `c2_verify_arrangement_rule_katakana_middle_dot`: U+30FB only between Han/Kana.
fn rule_katakana_middle_dot(cps: &[u32]) -> Result<(), FacrError> {
    for i in 0..cps.len() {
        if cps[i] == 0x30FB {
            if i == 0 || i == cps.len() - 1 {
                return Err(FacrError::KatakanaMiddleDotContext);
            }
            let han_or_kana = |cp| tables::contains(tables::FLT03_JAPANESE_NATIVE, cp);
            if !han_or_kana(cps[i - 1]) || !han_or_kana(cps[i + 1]) {
                return Err(FacrError::KatakanaMiddleDotContext);
            }
        }
    }
    Ok(())
}

/// `c2_verify_arrangement_rule_native`: at least one character of the category's
/// own writing system. (Employability is checked first, so an absent script can
/// only mean "not native".)
fn rule_native(cps: &[u32], native: &[(u32, u32)]) -> Result<(), FacrError> {
    if cps.iter().any(|&cp| tables::contains(native, cp)) {
        Ok(())
    } else {
        Err(FacrError::MissingNativeCharacter)
    }
}

/// `c2_verify_arrangement_rule_decimal_digits`: all decimal digits in the name
/// belong to one numbering system (FLT11 range id).
fn rule_single_decimal_system(cps: &[u32]) -> Result<(), FacrError> {
    let mut first = None;
    for &cp in cps {
        if let Some(range) = tables::lookup_scalar(tables::FLT11_DECIMAL_RANGE, cp) {
            match first {
                None => first = Some(range),
                Some(f) if f != range => return Err(FacrError::MixedDecimalSystems),
                Some(_) => {}
            }
        }
    }
    Ok(())
}

/// Site coupling: if both the site and its network use a connector, it must be
/// the same connector character.
fn rule_connector_matches_network(network: &[u32], site: &[u32]) -> Result<(), FacrError> {
    let net = connectors_used(network);
    let sit = connectors_used(site);
    if !net.is_empty() && !sit.is_empty() && net != sit {
        return Err(FacrError::ConnectorMismatchWithNetwork);
    }
    Ok(())
}

/// Site coupling: if both the site and its network use decimal digits, they must
/// be from the same numbering system.
fn rule_decimal_matches_network(network: &[u32], site: &[u32]) -> Result<(), FacrError> {
    let net = decimal_systems_used(network);
    let sit = decimal_systems_used(site);
    if !net.is_empty() && !sit.is_empty() && net != sit {
        return Err(FacrError::DecimalSystemMismatchWithNetwork);
    }
    Ok(())
}

/// The distinct connector characters appearing in a name (a singleton wherever
/// the single-connector rule applies).
fn connectors_used(cps: &[u32]) -> Vec<u32> {
    let mut seen = Vec::new();
    for &cp in cps {
        if CONNECTORS.contains(&cp) && !seen.contains(&cp) {
            seen.push(cp);
        }
    }
    seen.sort_unstable();
    seen
}

/// The distinct decimal numbering-system ids (FLT11) appearing in a name.
fn decimal_systems_used(cps: &[u32]) -> Vec<u32> {
    let mut seen = Vec::new();
    for &cp in cps {
        if let Some(range) = tables::lookup_scalar(tables::FLT11_DECIMAL_RANGE, cp)
            && !seen.contains(&range)
        {
            seen.push(range);
        }
    }
    seen.sort_unstable();
    seen
}

// ---- Network-name convergence (FACR 1.0 §8) ---------------------------------

/// Whether two valid network names are convergent, per the full FACR 1.0 §8
/// method — including the overlapping-linguistic-category cases.
///
/// Each address carries its own category (`lc_a`, `lc_b`); the names are assumed
/// to be valid network names for those categories. Two networks converge if they
/// are IFAP-identical, share an Intra-LC convergence form under their common or
/// an overlapping category, or (across categories) share the Inter-LC form.
///
/// The overlap reasoning is folded in directly: a candidate third category `LCo`
/// "overlaps" only where both names are valid network names for it, so the cases
/// that require overlap become vacuously false otherwise and need no separate
/// category-overlap table.
pub fn network_names_converge(
    a: &FrogansAddress,
    lc_a: LinguisticCategory,
    b: &FrogansAddress,
    lc_b: LinguisticCategory,
) -> bool {
    // A. Identical network names according to IFAP.
    if network_reference(a) == network_reference(b) {
        return true;
    }

    let nn1 = a.network_name();
    let nn2 = b.network_name();

    if lc_a == lc_b {
        // B.1. Shared Intra-LC form under the common category.
        if intra_lc_match(lc_a, nn1, nn2) {
            return true;
        }
        // B.2. Shared Intra-LC form under an overlapping category LCo.
        return LinguisticCategory::ALL.into_iter().any(|lco| {
            lco != lc_a
                && a.is_valid_network_name(lco)
                && b.is_valid_network_name(lco)
                && intra_lc_match(lco, nn1, nn2)
        });
    }

    // C. Different categories.
    // C.2.i / C.2.ii: one name is also valid for the other's category and they
    // share that category's Intra-LC form. (Vacuously false without overlap.)
    if b.is_valid_network_name(lc_a) && intra_lc_match(lc_a, nn1, nn2) {
        return true;
    }
    if a.is_valid_network_name(lc_b) && intra_lc_match(lc_b, nn1, nn2) {
        return true;
    }
    // C.1.i / C.2.iii: a third overlapping category LCo.
    let via_overlap = LinguisticCategory::ALL.into_iter().any(|lco| {
        lco != lc_a
            && lco != lc_b
            && a.is_valid_network_name(lco)
            && b.is_valid_network_name(lco)
            && intra_lc_match(lco, nn1, nn2)
    });
    if via_overlap {
        return true;
    }
    // C.1.ii / C.2.iv: the Inter-LC form.
    inter_lc_convergence_form(nn1) == inter_lc_convergence_form(nn2)
}

/// Whether two names share an Intra-LC convergence form of any type under `lc`.
fn intra_lc_match(lc: LinguisticCategory, a_name: &str, b_name: &str) -> bool {
    let a_forms = lc.intra_lc_convergence_forms(a_name);
    let b_forms = lc.intra_lc_convergence_forms(b_name);
    a_forms.iter().any(|form| b_forms.contains(form))
}
