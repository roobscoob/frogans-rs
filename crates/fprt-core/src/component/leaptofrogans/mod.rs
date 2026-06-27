//! `leaptofrogans` — the "Leap to Frogans" confirmation dialog payloads (commands
//! only; its five events and the lifecycle commands are no-data markers handled
//! by the transport layer).

mod cmd_update_address;
mod cmd_update_labels;

pub use cmd_update_address::UpdateAddress;
pub use cmd_update_labels::UpdateLabels;
