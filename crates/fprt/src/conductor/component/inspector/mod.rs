//! `inspector` — the per-site run-inspector dialog.
//!
//! **Multi-instance**: every command and event carries an [`InspectorId`] naming
//! which window it targets. So unlike the other dialogs, inspector has no bare
//! markers — even open/show/hide and the value-less events thread the id. The two
//! macros below are the local equivalents of the shared
//! [`marker_command!`](crate::conductor::command::marker_command) /
//! [`marker_event!`](crate::conductor::report::marker_event), with the id woven
//! in.

use fprt_sys::ui::inspector::head::Head;
use fprt_sys::ui::inspector::ref_event::RefEvent;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::call::invoke;
use crate::conductor::command::{Command, CommandPayload};
use crate::conductor::report::{sealed, Report};
use crate::conductor::Conductor;
use crate::error::EngineError;
use crate::pool::Pool;

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

/// A lifecycle command (open/show/hide/push/close) — the engine drives it at one
/// window; the whole payload is the [`Head`], so we surface just the id.
macro_rules! inspector_marker {
    ($name:ident, $id:expr, $export:ident, $variant:ident) => {
        /// A lifecycle command targeting one inspector window.
        pub struct $name(pub InspectorId);

        impl CommandPayload for $name {
            const ID: StatusName = $id;
            type Raw = Head;

            fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
                methods.$export
            }

            fn from_raw(raw: Head, _pool: &Pool) -> Self {
                $name(InspectorId(raw.reference))
            }

            fn into_command(self) -> Command {
                Command::$variant(self.0)
            }
        }
    };
}

/// A value-less event (synchronize/rerun/close) — reports only *which* window the
/// signal came from (the shared [`RefEvent`] payload).
macro_rules! ref_event {
    ($name:ident, $tag:expr, $export:ident) => {
        /// A value-less host→engine event from one inspector window.
        pub struct $name {
            id: InspectorId,
        }

        impl $name {
            /// The window the event came from.
            pub fn new(id: InspectorId) -> Self {
                $name { id }
            }
        }

        impl sealed::Sealed for $name {}

        impl Report for $name {
            fn send(self, conductor: &Conductor) -> Result<(), EngineError> {
                let engine = conductor.engine();
                let ctx = conductor.ctx();
                let payload = RefEvent {
                    event_id: $tag,
                    inspector_ref: self.id.0,
                };
                // SAFETY: valid ctx; `payload` outlives the call.
                invoke(engine, |s, e, p| unsafe {
                    (engine.methods().$export)(ctx, &payload, s, e, p)
                })
                .map(|_| ())
            }
        }
    };
}

inspector_marker!(Open, fprt_sys::ui::inspector::CMD_OPEN, inspector_open, InspectorOpen);
inspector_marker!(Show, fprt_sys::ui::inspector::CMD_SHOW, inspector_show, InspectorShow);
inspector_marker!(Hide, fprt_sys::ui::inspector::CMD_HIDE, inspector_hide, InspectorHide);
inspector_marker!(Push, fprt_sys::ui::inspector::CMD_PUSH, inspector_push, InspectorPush);
inspector_marker!(Close, fprt_sys::ui::inspector::CMD_CLOSE, inspector_close, InspectorClose);

ref_event!(ReportSynchronize, fprt_sys::ui::inspector::EVT_SYNCHRONIZE, inspector_synchronize);
ref_event!(ReportRerun, fprt_sys::ui::inspector::EVT_RERUN, inspector_rerun);
ref_event!(ReportClose, fprt_sys::ui::inspector::EVT_CLOSE, inspector_close_event);
