//! `legalinformation` — the legal-information / OSS-license panel payloads.
//!
//! Its lifecycle commands (open/show/push/hide/close) and `close` event are
//! no-data markers, handled by the transport layer, so only the two data
//! payloads live here. `update_legal_content` is a nested document tree, ported
//! decode-only (see [`cmd_update_legal_content`]).

mod cmd_update_labels;
mod cmd_update_legal_content;

pub use cmd_update_labels::UpdateLabels;
pub use cmd_update_legal_content::{Document, LegalContentKind, Topic, UpdateLegalContent};
