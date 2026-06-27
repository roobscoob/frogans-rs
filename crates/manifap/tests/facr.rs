//! FACR layer: employable characters, convergence-form generation, and the
//! FACR 1.1 site-name convergence test.

use manifap::FrogansAddress;
use manifap::facr::{self, FacrError, LinguisticCategory as LC};

fn eval(s: &str) -> FrogansAddress {
    FrogansAddress::evaluate(s).unwrap_or_else(|e| panic!("`{s}` should be valid: {e}"))
}

#[test]
fn employable_characters_are_category_relative() {
    assert!(LC::Latin.employable_characters_ok("test"));
    let cyrillic = "\u{0442}\u{0435}\u{0441}\u{0442}"; // тест
    assert!(!LC::Latin.employable_characters_ok(cyrillic));
    assert!(LC::Cyrillic.employable_characters_ok(cyrillic));
}

#[test]
fn labels_round_trip() {
    for lc in LC::ALL {
        assert_eq!(LC::from_label(lc.label()), Some(lc));
    }
    assert_eq!(LC::from_label("LC-Nonsense"), None);
}

#[test]
fn chinese_has_two_intra_lc_types_others_have_one() {
    let forms = LC::Chinese.intra_lc_convergence_forms("test");
    assert_eq!(forms.len(), 2);
    assert_eq!(forms[0].type_label(), "Intra-LC-Chinese-Confusable");
    assert_eq!(forms[1].type_label(), "Intra-LC-Chinese-Variant");

    assert_eq!(LC::Latin.intra_lc_convergence_forms("test").len(), 1);
}

#[test]
fn latin_confusable_skeleton_collapses_lookalikes() {
    // FLT12 maps U+0049 LATIN CAPITAL I and U+004C LATIN CAPITAL L to U+0031
    // DIGIT ONE, so "I", "L", and "1" share a confusable skeleton.
    let forms = |name| LC::Latin.intra_lc_convergence_forms(name);
    let i = forms("I");
    let l = forms("L");
    let one = forms("1");
    assert_eq!(i, one);
    assert_eq!(l, one);
    assert_eq!(i[0].value(), "1");
}

#[test]
fn diacritics_are_preserved_in_skeletons() {
    // FACR §7 (citing UTR#36): "n" and "ñ" must NOT collapse together.
    let n = LC::Latin.intra_lc_convergence_forms("n");
    let n_tilde = LC::Latin.intra_lc_convergence_forms("\u{00F1}"); // ñ
    assert_ne!(n, n_tilde);
}

#[test]
fn site_names_converge_on_confusable() {
    // Same network, Latin. Site names "I" and "1" are not IFAP-identical
    // (reference forms "i" vs "1") but share a confusable skeleton.
    let a = eval("shop*I");
    let b = eval("shop*1");
    assert_ne!(a, b); // not identical
    assert!(facr::site_names_converge(LC::Latin, &a, &b));
}

#[test]
fn site_names_converge_when_identical() {
    // Case-only difference: IFAP-identical, hence convergent (test A).
    let a = eval("shop*Sale");
    let b = eval("shop*SALE");
    assert!(facr::site_names_converge(LC::Latin, &a, &b));
}

#[test]
fn distinct_site_names_do_not_converge() {
    let a = eval("shop*apple");
    let b = eval("shop*orange");
    assert!(!facr::site_names_converge(LC::Latin, &a, &b));
}

#[test]
fn network_forms_include_inter_lc_but_site_forms_do_not() {
    let addr = eval("test*site");
    let network = addr.network_convergence_forms(LC::Latin);
    let site = addr.site_convergence_forms(LC::Latin);
    assert!(network.iter().any(|f| f.type_label() == "Inter-LC"));
    assert!(!site.iter().any(|f| f.type_label() == "Inter-LC"));
}

// ---- arrangement rules + validity -------------------------------------------

#[test]
fn plain_latin_name_is_valid() {
    let a = eval("my-network*my-site");
    assert!(a.is_valid_network_name(LC::Latin));
    assert!(a.is_valid_site_name(LC::Latin)); // same connector in both
}

#[test]
fn middle_dot_must_sit_between_two_els() {
    // Catalan "l·l" is allowed.
    let ok = eval("paral\u{00B7}lel*test");
    assert!(ok.is_valid_network_name(LC::Latin));
    // Anywhere else it is rejected with a precise error.
    let bad = eval("a\u{00B7}b*test");
    assert_eq!(
        bad.validate_network_name(LC::Latin),
        Err(FacrError::MiddleDotContext)
    );
}

#[test]
fn chinese_requires_a_native_han_character() {
    // U+4E00 and U+4E03 are Han (native to LC-Chinese).
    let han = eval("\u{4E00}\u{4E03}*\u{4E00}\u{4E03}");
    assert!(han.is_valid_network_name(LC::Chinese));
    assert!(han.is_valid_site_name(LC::Chinese));

    // A Latin-only name is a valid Latin name but not a valid Chinese one
    // (no native character / not employable).
    let latin = eval("test*test");
    assert!(latin.is_valid_network_name(LC::Latin));
    assert!(!latin.is_valid_network_name(LC::Chinese));
}

#[test]
fn validate_reports_rich_errors_not_just_a_bool() {
    let latin = eval("test*test");
    // The error is a typed reason, not a bare false.
    assert!(matches!(
        latin.validate_network_name(LC::Chinese),
        Err(FacrError::NotEmployable { .. } | FacrError::MissingNativeCharacter)
    ));
    assert!(latin.validate_network_name(LC::Latin).is_ok());
}

#[test]
fn categories_without_arrangement_rules_accept_freely() {
    // LC-Greek has no arrangement rules; any employable Greek name is valid.
    let greek = eval("\u{03B1}\u{03B2}\u{03B3}*\u{03B4}\u{03B5}"); // αβγ*δε
    assert!(greek.is_valid_network_name(LC::Greek));
    assert!(greek.is_valid_site_name(LC::Greek));
}

// ---- network-name convergence (§8) ------------------------------------------

#[test]
fn identical_network_names_converge_across_any_categories() {
    // Case A: identity short-circuits regardless of category.
    let a = eval("abc*x");
    let b = eval("abc*y");
    assert!(facr::network_names_converge(&a, LC::Latin, &b, LC::Greek));
}

#[test]
fn same_category_confusable_networks_converge() {
    // "Il" and "li" both skeletonize to "11" under LC-Latin, but are not
    // IFAP-identical ("il" vs "li").
    let a = eval("Il*x");
    let b = eval("li*x");
    assert_ne!(a.canonicalize().network_name(), b.canonicalize().network_name());
    assert!(facr::network_names_converge(&a, LC::Latin, &b, LC::Latin));
}

#[test]
fn distinct_networks_do_not_converge() {
    let a = eval("apple*x");
    let b = eval("orange*x");
    assert!(!facr::network_names_converge(&a, LC::Latin, &b, LC::Latin));
}
