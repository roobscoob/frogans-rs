//! `update` — the software-update dialog payloads (commands only; its `cancel`
//! event and the lifecycle commands are no-data markers handled by the transport
//! layer).

mod cmd_update_data;
mod cmd_update_labels;

pub use cmd_update_data::UpdateData;
pub use cmd_update_labels::UpdateLabels;
