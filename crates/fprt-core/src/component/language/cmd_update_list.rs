//! `update_list` command (engine → host) — the selectable languages.
//!
//! It carries a pooled array of multi-field [`Language`] records (`LangEntry`,
//! stride `0x20`): reading them is `from_raw`, building them into a pool is
//! `to_raw`/`new`.

use fprt_sys::ui::language::CMD_UPDATE_LIST;
use fprt_sys::ui::language::update_list::{LangEntry, UpdateList as Raw};

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

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

impl Language {
    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Language {
            identifier: pool.clone_str_opt(&self.identifier),
            name: pool.clone_str_opt(&self.name),
        }
    }
}

impl UpdateList {
    /// Build one, allocating each string into `pool`. Each tuple is
    /// `(identifier, name)`; an empty `current` stores `None`.
    pub fn new(pool: &OwnedPool, languages: &[(&str, &str)], current: &str) -> Self {
        let languages = languages
            .iter()
            .map(|(id, name)| Language {
                identifier: Some(pool.alloc_str(id)),
                name: Some(pool.alloc_str(name)),
            })
            .collect();
        let current = if current.is_empty() {
            None
        } else {
            Some(pool.alloc_str(current))
        };
        UpdateList { languages, current }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
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

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateList {
            languages: self.languages.iter().map(|l| l.copy_into(pool)).collect(),
            current: pool.clone_str_opt(&self.current),
        }
    }

    /// Encode into the raw payload, allocating the entry array into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        let (count, entries) = if self.languages.is_empty() {
            (0, core::ptr::null())
        } else {
            let records: Vec<LangEntry> = self
                .languages
                .iter()
                .map(|lang| LangEntry {
                    identifier: ustring_opt(lang.identifier.as_ref()),
                    language: ustring_opt(lang.name.as_ref()),
                })
                .collect();
            (
                self.languages.len() as u32,
                pool.alloc_slice(&records).cast::<LangEntry>(),
            )
        };
        Raw {
            status_id: CMD_UPDATE_LIST,
            _rsv04: 0,
            count,
            entries,
            current_lang_id: ustring_opt(self.current.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = UpdateList::new(
            &pool,
            &[("en", "English"), ("fr", "Français"), ("ja", "日本語")],
            "fr",
        );
        let back = UpdateList::from_raw(cmd.to_raw(&pool), &pool.as_pool());
        let ids: Vec<&str> = back
            .languages
            .iter()
            .map(|l| l.identifier.as_ref().unwrap().as_str().unwrap())
            .collect();
        let names: Vec<&str> = back
            .languages
            .iter()
            .map(|l| l.name.as_ref().unwrap().as_str().unwrap())
            .collect();
        assert_eq!(ids, ["en", "fr", "ja"]);
        assert_eq!(names, ["English", "Français", "日本語"]);
        assert_eq!(back.current.as_ref().unwrap().as_str().unwrap(), "fr");
    }

    #[test]
    fn empty_list_encodes_null() {
        let pool = OwnedPool::new();
        let raw = UpdateList {
            languages: vec![],
            current: None,
        }
        .to_raw(&pool);
        assert!(raw.entries.is_null());
        assert_eq!(raw.count, 0);
    }
}
