//! [`FrogansAddressEvaluationError`] — why a candidate string is not a Frogans
//! address. Each variant cites the IFAP section it comes from.

use core::fmt;

/// The network name or the site name half of a Frogans address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    /// The part before the separator (identifies a Frogans network).
    NetworkName,
    /// The part after the separator (identifies a site within the network).
    SiteName,
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Part::NetworkName => "network name",
            Part::SiteName => "site name",
        })
    }
}

/// A broken structural rule for a network/site name (IFAP §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StructureViolation {
    /// The part contains a second separator (`*`).
    ContainsSeparator,
    /// The first character is a combining mark (forbidden in both parts).
    FirstCharacterCombiningMark,
    /// The first character is a decimal number (forbidden in a network name).
    FirstCharacterDecimalNumber,
    /// The first character is one of the five signs barred from a network name's
    /// first position (U+0375, U+05F3, U+05F4, U+06FD, U+06FE).
    FirstCharacterDisallowed,
    /// A connector character is the first or last character of the part.
    ConnectorAtEdge,
    /// Two connector characters appear in a row.
    ConsecutiveConnectors,
    /// A connector character is immediately followed by a combining mark.
    ConnectorFollowedByCombiningMark,
}

impl fmt::Display for StructureViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StructureViolation::ContainsSeparator => "contains a second separator",
            StructureViolation::FirstCharacterCombiningMark => {
                "starts with a combining mark"
            }
            StructureViolation::FirstCharacterDecimalNumber => "starts with a decimal number",
            StructureViolation::FirstCharacterDisallowed => "starts with a disallowed sign",
            StructureViolation::ConnectorAtEdge => "starts or ends with a connector character",
            StructureViolation::ConsecutiveConnectors => "has consecutive connector characters",
            StructureViolation::ConnectorFollowedByCombiningMark => {
                "has a connector character followed by a combining mark"
            }
        })
    }
}

/// Why [`FrogansAddress::evaluate`](crate::FrogansAddress::evaluate) rejected a
/// candidate string.
///
/// Indices, where present, are **code-point** offsets into the candidate string
/// (not byte offsets), counting the separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FrogansAddressEvaluationError {
    /// The candidate, or one of its two parts, is empty.
    Empty,
    /// The candidate does not contain exactly one separator (`*`) character.
    SeparatorCount {
        /// How many separators were found (anything other than 1 is invalid).
        found: usize,
    },
    /// The separator is the first or the last character of the candidate.
    SeparatorAtEdge,
    /// A code point lies outside the Frogans address character set (IFAP §3.1).
    OutsideCharacterSet {
        /// Code-point index of the offending character.
        index: usize,
        /// The offending code point.
        code_point: u32,
    },
    /// The candidate is not stable under NFKC normalization (IFAP §3.2).
    NotNormalized,
    /// More than 30 consecutive combining marks (IFAP §3.2).
    ExcessiveCombiningMarks,
    /// A ZERO WIDTH (NON-)JOINER appears without its required joining/virama
    /// context (IFAP §3.2).
    ContextualJoiner {
        /// Code-point index of the joiner.
        index: usize,
        /// The joiner code point (U+200C or U+200D).
        code_point: u32,
    },
    /// A code point is not an eligible character (IFAP §3.3).
    Ineligible {
        /// Code-point index of the offending character.
        index: usize,
        /// The offending code point.
        code_point: u32,
    },
    /// The candidate violates the directionality / bidi rules (IFAP §3.4).
    Directionality,
    /// A network/site name breaks a structural rule (IFAP §4).
    Structure {
        /// Which part broke the rule.
        part: Part,
        /// The rule that was broken.
        violation: StructureViolation,
    },
    /// A part's reference-form length is outside the 1..=28 range (IFAP §6).
    Length {
        /// Which part is mis-sized.
        part: Part,
        /// The part's length, in characters of its reference form.
        characters: usize,
    },
}

impl fmt::Display for FrogansAddressEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use FrogansAddressEvaluationError as E;
        match self {
            E::Empty => write!(f, "the Frogans address, or one of its parts, is empty"),
            E::SeparatorCount { found } => {
                write!(f, "expected exactly one separator '*', found {found}")
            }
            E::SeparatorAtEdge => write!(f, "the separator '*' cannot be the first or last character"),
            E::OutsideCharacterSet { index, code_point } => write!(
                f,
                "U+{code_point:04X} at position {index} is outside the Frogans address character set"
            ),
            E::NotNormalized => write!(f, "the string is not in NFKC normalized form"),
            E::ExcessiveCombiningMarks => {
                write!(f, "more than 30 consecutive combining marks")
            }
            E::ContextualJoiner { index, code_point } => write!(
                f,
                "U+{code_point:04X} at position {index} lacks the required joining context"
            ),
            E::Ineligible { index, code_point } => write!(
                f,
                "U+{code_point:04X} at position {index} is not an eligible character"
            ),
            E::Directionality => write!(f, "the string violates the directionality rules"),
            E::Structure { part, violation } => write!(f, "the {part} {violation}"),
            E::Length { part, characters } => write!(
                f,
                "the {part} is {characters} characters long; it must be between 1 and 28"
            ),
        }
    }
}

impl std::error::Error for FrogansAddressEvaluationError {}
