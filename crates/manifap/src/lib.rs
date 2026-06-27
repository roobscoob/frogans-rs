//! Frogans address evaluation and canonicalization.
//!
//! A *Frogans address* identifies a Frogans site. It is written `network*site`:
//! a network name, the `*` separator, and a site name (right-to-left scripts put
//! the network name on the right). Two specifications govern it, and this crate
//! implements both, pinned to **Unicode 7.0.0** via bundled, hash-verified
//! lookup tables — not a general Unicode crate, whose newer data would disagree.
//!
//! * **IFAP** ([`FrogansAddress::evaluate`]) — the address *pattern*: which code
//!   points are allowed, NFKC formation, directionality, structure, length, and
//!   the case-folded *reference form* that decides when two addresses are the
//!   same.
//! * **FACR** ([`facr`]) — the *composition rules* the registry layers on top to
//!   stop look-alike/confusable registrations: per–linguistic-category employable
//!   characters and convergence forms. FACR is defined only within a declared
//!   linguistic category, so it lives in its own module rather than on the type.
//!
//! # Quick start
//!
//! ```
//! use manifap::FrogansAddress;
//!
//! let a = FrogansAddress::evaluate("MyNetwork*MySite").unwrap();
//! let b = FrogansAddress::evaluate("mynetwork*mysite").unwrap();
//!
//! // Identity is by reference form (IFAP §7): case folds away.
//! assert_eq!(a, b);
//! assert_eq!(a.reference_form(), "mynetwork*mysite");
//!
//! // The preferred form is preserved for display.
//! assert_eq!(a.as_str(), "MyNetwork*MySite");
//! assert!(a.canonicalize().is_canonical());
//! ```

mod address;
mod error;
mod ifap;
mod normalize;
mod tables;

pub mod facr;

pub use address::FrogansAddress;
pub use error::{FrogansAddressEvaluationError, Part, StructureViolation};
