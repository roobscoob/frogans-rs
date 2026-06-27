//! `inputfa` — the input-Frogans-Address text-entry dialog.
//!
//! `update_labels` / `update_address` / `update_error_raise` are pooled-string
//! commands; `change` / `ok` are borrowed-string events. All full codecs. The
//! bare signal commands and the `cancel` event are markers handled by transport.

mod cmd_update_address;
mod cmd_update_error_raise;
mod cmd_update_labels;
mod evt_change;
mod evt_ok;

pub use cmd_update_address::UpdateAddress;
pub use cmd_update_error_raise::UpdateErrorRaise;
pub use cmd_update_labels::UpdateLabels;
pub use evt_change::ReportChange;
pub use evt_ok::ReportOk;
