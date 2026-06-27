//! `zoom` — the zoom-settings dialog payloads. Lifecycle commands and the
//! `cancel` event are no-data markers handled by the transport layer.

mod cmd_update_labels;
mod evt_ok;

pub use cmd_update_labels::UpdateLabels;
pub use evt_ok::ReportOk;
