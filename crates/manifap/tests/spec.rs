//! Conformance tests drawn from the IFAP/FACR worked examples plus the identity,
//! canonicalization, and rejection behavior of the public API.

use std::collections::HashSet;

use manifap::facr::LinguisticCategory;
use manifap::{FrogansAddress, FrogansAddressEvaluationError as E, Part, StructureViolation as S};

fn eval(s: &str) -> FrogansAddress {
    FrogansAddress::evaluate(s).unwrap_or_else(|e| panic!("`{s}` should be valid: {e}"))
}

fn err(s: &str) -> E {
    FrogansAddress::evaluate(s).expect_err(&format!("`{s}` should be rejected"))
}

#[test]
fn basic_valid_address() {
    let a = eval("test*site");
    assert_eq!(a.as_str(), "test*site");
    assert_eq!(a.reference_form(), "test*site");
    assert!(a.is_canonical());
}

#[test]
fn ifap_section7_identity_examples() {
    // From IFAP §7: these are all the same address; reference form is lowercase.
    let reference = "mynetwork*mysite";
    for variant in ["mynetwork*mysite", "MyNetwork*MYSITE", "MYNETWORK*MySite"] {
        let a = eval(variant);
        assert_eq!(a.reference_form(), reference, "{variant}");
        assert_eq!(a, eval(reference), "{variant}");
    }

    // ...but a connector character is significant: these are NOT identical.
    assert_ne!(eval("my-network*MySite"), eval("mynetwork*MySite"));
}

#[test]
fn eszett_folds_to_ss() {
    // IFAP §5 worked example: ß (U+00DF) case-folds to "ss".
    let a = eval("gro\u{00DF}*site"); // "groß*site"
    assert_eq!(a.reference_form(), "gross*site");
    assert_eq!(a, eval("gross*site"));
    // Preferred form is untouched.
    assert_eq!(a.as_str(), "gro\u{00DF}*site");
}

#[test]
fn case_fold_identity_hashes_equal() {
    let mut set = HashSet::new();
    set.insert(eval("MyNetwork*MySite"));
    // Equal addresses must collide in a HashSet (Hash agrees with Eq).
    assert!(set.contains(&eval("mynetwork*mysite")));
    assert!(!set.contains(&eval("othernet*site")));
}

#[test]
fn canonicalize_is_idempotent_and_equal() {
    let a = eval("MyNetwork*MySite");
    let c = a.canonicalize();
    assert_eq!(a, c);
    assert!(c.is_canonical());
    assert_eq!(c.as_str(), "mynetwork*mysite");
    assert_eq!(c.canonicalize(), c);
    // The reference form is itself a valid, self-canonical address.
    assert_eq!(eval(a.reference_form()), c);
}

#[test]
fn site_name_may_start_with_a_digit_but_network_may_not() {
    eval("net*1site"); // site: digit-first allowed (§4.3)

    // A network may not start with a decimal number (§4.2). An ASCII digit is
    // *also* bidi-EN, and IFAP checks directionality (§3.4) before structure
    // (§4), so the first reported failure is directionality.
    assert_eq!(err("1net*site"), E::Directionality);

    // A Devanagari digit (U+0966, bidi-L) clears directionality and is then
    // caught by the structure rule itself.
    assert_eq!(
        err("\u{0966}\u{0928}*site"),
        E::Structure {
            part: Part::NetworkName,
            violation: S::FirstCharacterDecimalNumber,
        }
    );
}

#[test]
fn separator_rules() {
    assert_eq!(err(""), E::Empty);
    assert_eq!(err("nostar"), E::SeparatorCount { found: 0 });
    assert_eq!(err("a*b*c"), E::SeparatorCount { found: 2 });
    // A leading/trailing `*` is bidi-neutral (ON), so §3.4 directionality
    // rejects it before the §4.1 "separator at edge" rule is reached.
    assert_eq!(err("*site"), E::Directionality);
    assert_eq!(err("net*"), E::Directionality);
}

#[test]
fn connector_rules() {
    // A single interior connector is fine...
    eval("my-network*my-site");

    // ...two in a row is a structural violation (reachable: hyphen is bidi-ES,
    // allowed in the interior of an LTR string).
    assert_eq!(
        err("ne--t*site"),
        E::Structure {
            part: Part::NetworkName,
            violation: S::ConsecutiveConnectors
        }
    );

    // A connector immediately followed by a combining mark (U+0301) is rejected.
    assert_eq!(
        err("ne-\u{0301}t*site"),
        E::Structure {
            part: Part::NetworkName,
            violation: S::ConnectorFollowedByCombiningMark
        }
    );

    // An edge connector ('-' is bidi-ES/-neutral) trips directionality first.
    assert_eq!(err("-net*site"), E::Directionality);
    assert_eq!(err("net*site-"), E::Directionality);
}

#[test]
fn length_bounds() {
    let long = "a".repeat(29);
    match err(&format!("{long}*site")) {
        E::Length {
            part: Part::NetworkName,
            characters,
        } => assert_eq!(characters, 29),
        other => panic!("expected length error, got {other:?}"),
    }
    // Exactly 28 is allowed.
    eval(&format!("{}*site", "a".repeat(28)));
}

#[test]
fn rejects_non_nfkc_input() {
    // "cafe" + COMBINING ACUTE ACCENT is not NFKC (it would compose to "café").
    assert_eq!(err("cafe\u{0301}*site"), E::NotNormalized);
    // The composed form is accepted and folds back through NFD/NFC.
    let a = eval("caf\u{00E9}*site");
    assert_eq!(a.reference_form(), "caf\u{00E9}*site");
}

#[test]
fn rejects_ineligible_character() {
    // U+0020 SPACE is in the character set but not an eligible character.
    match err("a b*site") {
        E::Ineligible { index, code_point } => {
            assert_eq!((index, code_point), (1, 0x20));
        }
        other => panic!("expected ineligible, got {other:?}"),
    }
}

#[test]
fn rejects_mixed_directionality() {
    // Latin 'a' (L) followed by Arabic letters (AL) breaks the LTR bidi rule.
    assert_eq!(err("a\u{0645}\u{062D}*site"), E::Directionality);
}

#[test]
fn rtl_address_is_accepted() {
    // An all-Arabic network and site (right-to-left) should validate.
    eval("\u{0645}\u{062D}\u{0645}\u{062F}*\u{0645}\u{0648}\u{0642}\u{0639}");
}

#[test]
fn facr_employable_characters() {
    assert!(LinguisticCategory::Latin.employable_characters_ok("test"));
    // Cyrillic "тест" is not employable in LC-Latin...
    assert!(!LinguisticCategory::Latin.employable_characters_ok("\u{0442}\u{0435}\u{0441}\u{0442}"));
    // ...but is in LC-Cyrillic.
    assert!(
        LinguisticCategory::Cyrillic.employable_characters_ok("\u{0442}\u{0435}\u{0441}\u{0442}")
    );
    assert_eq!(LinguisticCategory::from_label("LC-Greek"), Some(LinguisticCategory::Greek));
}
