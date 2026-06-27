//! The public [`FrogansAddress`] type.

use core::fmt;
use core::hash::{Hash, Hasher};
use core::str::FromStr;

use crate::error::FrogansAddressEvaluationError;
use crate::ifap;

/// A validated Frogans address.
///
/// A value of this type is, by construction, a string that satisfies the entire
/// IFAP 1.1 pattern (character set, NFKC formation, eligible characters,
/// directionality, structure, and length). Build one with
/// [`evaluate`](FrogansAddress::evaluate); there is no other way to obtain one,
/// so holding a `FrogansAddress` is proof of validity.
///
/// # Two forms
///
/// Every address carries two strings:
///
/// * the **preferred form** — exactly what was evaluated, suitable for display
///   ([`as_str`](FrogansAddress::as_str) / [`AsRef<str>`]);
/// * the **reference form** — the case-folded canonical form
///   ([`reference_form`](FrogansAddress::reference_form)) used to decide
///   identity.
///
/// # Identity
///
/// Two addresses are equal exactly when their reference forms match, which is
/// the IFAP §7 definition of "identical". So `MyNetwork*Site` and
/// `mynetwork*site` compare equal and hash equally, even though their preferred
/// forms differ. [`Hash`] is consistent with [`Eq`] because it hashes only the
/// reference form.
#[derive(Clone)]
pub struct FrogansAddress {
    preferred: String,
    reference: String,
}

impl FrogansAddress {
    /// Evaluate a candidate string against IFAP 1.1.
    ///
    /// On success the returned address keeps `candidate` verbatim as its
    /// preferred form. On failure the [`FrogansAddressEvaluationError`] names the
    /// first rule that was broken.
    pub fn evaluate(candidate: &str) -> Result<Self, FrogansAddressEvaluationError> {
        let evaluation = ifap::evaluate(candidate)?;
        Ok(FrogansAddress {
            preferred: candidate.to_owned(),
            reference: evaluation.reference,
        })
    }

    /// Return the canonical (reference-form) address: an address equal to this
    /// one whose preferred form *is* the reference form.
    ///
    /// The result is idempotent — `a.canonicalize().canonicalize()` equals
    /// `a.canonicalize()` — and `a == a.canonicalize()` always holds.
    pub fn canonicalize(&self) -> FrogansAddress {
        FrogansAddress {
            preferred: self.reference.clone(),
            reference: self.reference.clone(),
        }
    }

    /// The preferred form: the string this address was evaluated from.
    pub fn as_str(&self) -> &str {
        &self.preferred
    }

    /// The network name (preferred form): the part before the separator.
    ///
    /// This is the logical part preceding `*`, which is correct for both
    /// left-to-right and right-to-left addresses. For the reference-form network
    /// name, use `self.canonicalize().network_name()`.
    pub fn network_name(&self) -> &str {
        self.preferred
            .split_once('*')
            .expect("a validated address has exactly one separator")
            .0
    }

    /// The site name (preferred form): the part after the separator.
    pub fn site_name(&self) -> &str {
        self.preferred
            .split_once('*')
            .expect("a validated address has exactly one separator")
            .1
    }

    /// The reference form: the case-folded canonical string used for identity
    /// (IFAP §5). Not intended for display to end users.
    pub fn reference_form(&self) -> &str {
        &self.reference
    }

    /// Whether the preferred form already equals the reference form, i.e. this
    /// address is its own canonical form.
    pub fn is_canonical(&self) -> bool {
        self.preferred == self.reference
    }
}

impl PartialEq for FrogansAddress {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
    }
}

impl Eq for FrogansAddress {}

impl Hash for FrogansAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reference.hash(state);
    }
}

impl AsRef<str> for FrogansAddress {
    fn as_ref(&self) -> &str {
        &self.preferred
    }
}

impl fmt::Display for FrogansAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.preferred)
    }
}

impl fmt::Debug for FrogansAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrogansAddress")
            .field("preferred", &self.preferred)
            .field("reference", &self.reference)
            .finish()
    }
}

impl FromStr for FrogansAddress {
    type Err = FrogansAddressEvaluationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FrogansAddress::evaluate(s)
    }
}
