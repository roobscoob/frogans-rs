//! `update_list` command (engine → host) — the selectable languages.

use fprt_sys::ui::language::update_list::UpdateList as Raw;
use fprt_sys::ui::language::CMD_UPDATE_LIST;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// One selectable interface language.
#[derive(Debug)]
pub struct Language {
    /// Language code/identifier.
    pub identifier: Option<PooledString>,
    /// Human-readable display name.
    pub name: Option<PooledString>,
}

/// The list of selectable languages plus the intended current selection.
#[derive(Debug)]
pub struct UpdateList {
    /// The selectable languages, in engine order.
    pub languages: Vec<Language>,
    /// The identifier the engine intends as the current selection (may be empty).
    pub current: Option<PooledString>,
}

impl CommandPayload for UpdateList {
    const ID: StatusName = CMD_UPDATE_LIST;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.language_update_list
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        let mut languages = Vec::with_capacity(raw.count as usize);
        if !raw.entries.is_null() {
            for i in 0..raw.count as usize {
                // SAFETY: `entries` points at `count` contiguous records (stride
                // 0x20), all written into `pool` by the pop that produced them.
                let entry = unsafe { *raw.entries.add(i) };
                let (identifier, name) =
                    unsafe { (pool.string(entry.identifier), pool.string(entry.language)) };
                languages.push(Language { identifier, name });
            }
        }
        // SAFETY: `current_lang_id` was written into `pool` by the same pop.
        let current = unsafe { pool.string(raw.current_lang_id) };
        UpdateList { languages, current }
    }

    fn into_command(self) -> Command {
        Command::LanguageUpdateList(self)
    }
}
