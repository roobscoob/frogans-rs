//! `update_directionality` command (engine → host) — the text-directionality enum.

use fprt_sys::ui::application::directionality::Directionality as RawDirectionality;
use fprt_sys::ui::application::update_directionality::UpdateDirectionality as Raw;
use fprt_sys::ui::application::CMD_UPDATE_DIRECTIONALITY;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

/// Text directionality. Only [`Default`](Directionality::Default) is labelled;
/// the other two values are proven but their LTR/RTL meaning is unresolved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Directionality {
    /// `0xbb8` — default.
    Default,
    /// `0xbb9` — host selection 1 (meaning unresolved).
    Selected1,
    /// `0xbba` — host selection 2 (meaning unresolved).
    Selected2,
    /// An engine value outside the documented set.
    Other(u32),
}

impl Directionality {
    fn from_raw(raw: RawDirectionality) -> Self {
        match raw {
            RawDirectionality::DEFAULT => Directionality::Default,
            RawDirectionality::SELECTED_1 => Directionality::Selected1,
            RawDirectionality::SELECTED_2 => Directionality::Selected2,
            _ => Directionality::Other(raw.0),
        }
    }
}

/// The text directionality the host should apply.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateDirectionality {
    /// The directionality enum.
    pub directionality: Directionality,
}

impl CommandPayload for UpdateDirectionality {
    const ID: StatusName = CMD_UPDATE_DIRECTIONALITY;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_directionality
    }

    fn from_raw(raw: Raw, _pool: &Pool) -> Self {
        UpdateDirectionality {
            directionality: Directionality::from_raw(raw.directionality),
        }
    }

    fn into_command(self) -> Command {
        Command::ApplicationUpdateDirectionality(self)
    }
}
