//! inspector — per-site run inspector dialog (`fprt_ui_inspector_*`).
//!
//! 12 commands + 6 events. **Multi-instance**: every payload begins with the
//! 8-byte [`Head`] carrying the inspector `reference` that selects which window.
//! 5 bare lifecycle (`Pop<Head>`), 7 data commands, 6 events. Command statuses
//! `0x18bxxxxx`, events `0x18aexxxx`.
//!
//! [`Head`]: head::Head

pub mod change_autosync;
pub mod content_selected;
pub mod head;
pub mod labels;
pub mod ref_event;
pub mod step_selected;
pub mod update_address;
pub mod update_content_labels;
pub mod update_content_viewer;
pub mod update_status;
pub mod update_steps_labels;
pub mod update_sync;

use crate::ui::{EventTag, Pop, Report, StatusName};

// --- command type tags ---
pub const CMD_OPEN: StatusName = StatusName(0x21961d);
pub const CMD_UPDATE_ADDRESS: StatusName = StatusName(0x21961e);
pub const CMD_UPDATE_STATUS: StatusName = StatusName(0x21961f);
pub const CMD_UPDATE_LABELS: StatusName = StatusName(0x219620);
pub const CMD_UPDATE_STEPS_LABELS: StatusName = StatusName(0x219621);
pub const CMD_UPDATE_CONTENT_LABELS: StatusName = StatusName(0x219622);
pub const CMD_UPDATE_CONTENT_VIEWER: StatusName = StatusName(0x219623);
pub const CMD_UPDATE_SYNC: StatusName = StatusName(0x219625);
pub const CMD_SHOW: StatusName = StatusName(0x219626);
pub const CMD_PUSH: StatusName = StatusName(0x219627);
pub const CMD_HIDE: StatusName = StatusName(0x219628);
pub const CMD_CLOSE: StatusName = StatusName(0x219629);

// --- event tags ---
pub const EVT_STEP_SELECTED: EventTag = EventTag(0x10ccf8);
pub const EVT_CONTENT_SELECTED: EventTag = EventTag(0x10ccf9);
pub const EVT_SYNCHRONIZE: EventTag = EventTag(0x10ccfa);
pub const EVT_CHANGE_AUTOSYNC: EventTag = EventTag(0x10ccfb);
pub const EVT_RERUN: EventTag = EventTag(0x10ccfc);
pub const EVT_CLOSE: EventTag = EventTag(0x10ccfd);

// --- the 18 calls ---
pub type OpenPop = Pop<head::Head>;
pub type ClosePop = Pop<head::Head>;
pub type ShowPop = Pop<head::Head>;
pub type HidePop = Pop<head::Head>;
pub type PushPop = Pop<head::Head>;
pub type UpdateAddressPop = Pop<update_address::UpdateAddress>;
pub type UpdateStatusPop = Pop<update_status::UpdateStatus>;
pub type UpdateLabelsPop = Pop<labels::Labels>;
pub type UpdateStepsLabelsPop = Pop<update_steps_labels::UpdateStepsLabels>;
pub type UpdateContentLabelsPop = Pop<update_content_labels::UpdateContentLabels>;
pub type UpdateContentViewerPop = Pop<update_content_viewer::UpdateContentViewer>;
pub type UpdateSyncPop = Pop<update_sync::UpdateSync>;
pub type StepSelectedReport = Report<step_selected::StepSelected>;
pub type ContentSelectedReport = Report<content_selected::ContentSelected>;
pub type SynchronizeReport = Report<ref_event::RefEvent>;
pub type ChangeAutosyncReport = Report<change_autosync::ChangeAutosync>;
pub type RerunReport = Report<ref_event::RefEvent>;
pub type CloseReport = Report<ref_event::RefEvent>;
