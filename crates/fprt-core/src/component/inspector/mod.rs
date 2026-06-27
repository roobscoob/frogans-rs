//! `inspector` — the per-site run-inspector dialog payloads.
//!
//! **Multi-instance**: every command and event carries an [`InspectorId`] naming
//! which window it targets — including the ref-events (synchronize/rerun/close),
//! which carry *only* the id (decoded by [`ref_event_id`]). The bare lifecycle
//! commands (open/show/hide/push/close) are markers handled by the transport layer,
//! so only the data payloads live here.

mod cmd_update_address;
mod cmd_update_content_labels;
mod cmd_update_content_viewer;
mod cmd_update_labels;
mod cmd_update_status;
mod cmd_update_steps_labels;
mod cmd_update_sync;
mod evt_change_autosync;
mod evt_content_selected;
mod evt_step_selected;

pub use cmd_update_address::UpdateAddress;
pub use cmd_update_content_labels::UpdateContentLabels;
pub use cmd_update_content_viewer::{ContentMode, UpdateContentViewer};
pub use cmd_update_labels::UpdateLabels;
pub use cmd_update_status::{RunStatus, UpdateStatus};
pub use cmd_update_steps_labels::UpdateStepsLabels;
pub use cmd_update_sync::UpdateSync;
pub use evt_change_autosync::ReportChangeAutosync;
pub use evt_content_selected::ReportContentSelected;
pub use evt_step_selected::ReportStepSelected;

/// Which inspector window a command targets / an event came from. The engine runs
/// one inspector per running site; this is the engine `data_subset` id (the host's
/// `_viewList` key). `Copy + Eq + Hash` so consumers can key a window map by it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InspectorId(pub i32);

/// Decode an inspector ref-event (synchronize / rerun / close) to the window id it
/// targets. These events carry no value beyond which inspector they came from.
pub fn ref_event_id(raw: &fprt_sys::ui::inspector::ref_event::RefEvent) -> InspectorId {
    InspectorId(raw.inspector_ref)
}
