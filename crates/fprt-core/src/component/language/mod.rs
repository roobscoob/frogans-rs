//! `language` — the interface-language selection dialog payloads. Lifecycle
//! commands and the `cancel` event are no-data markers handled by the transport
//! layer. [`UpdateList`] is decode-only for now (its encode lands with
//! `fprt-server`).

mod cmd_update_labels;
mod cmd_update_list;
mod evt_ok;

pub use cmd_update_labels::UpdateLabels;
pub use cmd_update_list::{Language, UpdateList};
pub use evt_ok::ReportOk;
