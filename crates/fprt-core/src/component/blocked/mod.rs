//! `blocked` — the blocked-addresses dialog payloads (commands only; its events
//! are no-data markers, handled by the transport layer).

mod cmd_update_addresses;
mod cmd_update_labels;

pub use cmd_update_addresses::UpdateAddresses;
pub use cmd_update_labels::UpdateLabels;
