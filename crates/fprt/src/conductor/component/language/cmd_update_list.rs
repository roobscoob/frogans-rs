//! `update_list` command (engine → host) — the selectable languages.

use fprt_sys::ui::language::update_list::UpdateList as Raw;
use fprt_sys::ui::language::CMD_UPDATE_LIST;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::Pool;

pub use fprt_core::component::language::{Language, UpdateList};

impl CommandPayload for UpdateList {
    const ID: StatusName = CMD_UPDATE_LIST;
    type Raw = Raw;
    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.language_update_list
    }
    fn decode(raw: Raw, pool: &Pool) -> Command {
        Command::LanguageUpdateList(UpdateList::from_raw(raw, pool))
    }
}
