//! `update_status` command (engine → host) — run outcome + data-available flag.

use fprt_sys::ui::inspector::CMD_UPDATE_STATUS;
use fprt_sys::ui::inspector::update_status::UpdateStatus as Raw;

use crate::component::inspector::InspectorId;

/// The outcome of a site run, as the inspector should display it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunStatus {
    /// Run completed (`0x3e9`).
    Completed,
    /// A run rejection was raised (`0x3ea`).
    RejectionRaised,
    /// An engine value outside the documented set.
    Other(u32),
}

impl RunStatus {
    /// Map the raw run-status word.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0x3e9 => RunStatus::Completed,
            0x3ea => RunStatus::RejectionRaised,
            other => RunStatus::Other(other),
        }
    }

    /// Map back to the raw run-status word.
    pub fn to_raw(self) -> u32 {
        match self {
            RunStatus::Completed => 0x3e9,
            RunStatus::RejectionRaised => 0x3ea,
            RunStatus::Other(v) => v,
        }
    }
}

/// The inspector's run status plus whether run data is available to show.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateStatus {
    /// The target window.
    pub id: InspectorId,
    /// The run outcome.
    pub run_status: RunStatus,
    /// `false` ⇒ show "run data not available".
    pub run_data_available: bool,
}

impl UpdateStatus {
    /// Build one (no pool — scalar/enum payload).
    pub fn new(id: InspectorId, run_status: RunStatus, run_data_available: bool) -> Self {
        UpdateStatus {
            id,
            run_status,
            run_data_available,
        }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateStatus {
            id: InspectorId(raw.reference),
            run_status: RunStatus::from_raw(raw.run_status),
            run_data_available: raw.run_data_available != 0,
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_STATUS,
            reference: self.id.0,
            run_status: self.run_status.to_raw(),
            run_data_available: self.run_data_available as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_every_variant() {
        for rs in [
            RunStatus::Completed,
            RunStatus::RejectionRaised,
            RunStatus::Other(0x1234),
        ] {
            for avail in [true, false] {
                let p = UpdateStatus::new(InspectorId(5), rs, avail);
                assert_eq!(UpdateStatus::from_raw(p.to_raw()), p);
            }
        }
    }
}
