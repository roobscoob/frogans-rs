//! `add_clipboard_text` command (engine → host) — a **pooled** payload.
//!
//! Reference command codec: one pooled string. Construct it (`new`) by allocating
//! into a pool, decode it (`from_raw`) by wrapping the engine's pooled bytes, and
//! encode it (`to_raw`) by pointing a descriptor back at those bytes.

use fprt_sys::ui::application::CMD_ADD_CLIPBOARD_TEXT;
use fprt_sys::ui::application::add_clipboard_text::AddClipboardText as Raw;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

/// Text the host must place on the system clipboard.
#[derive(Debug)]
pub struct AddClipboardText {
    /// The text, or `None` if empty.
    pub text: Option<PooledString>,
}

impl AddClipboardText {
    /// Build one, allocating `text` into `pool` (the producer / server side).
    pub fn new(pool: &OwnedPool, text: &str) -> Self {
        AddClipboardText {
            text: Some(pool.alloc_str(text)),
        }
    }

    /// Decode the engine's payload, wrapping its pooled bytes zero-copy (the
    /// consumer / client side).
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.text` was written into `pool` by the pop that produced both,
        // so its bytes live as long as the pool.
        AddClipboardText {
            text: unsafe { pool.string(raw.text) },
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        AddClipboardText {
            text: pool.clone_str_opt(&self.text),
        }
    }

    /// Encode into the raw payload, pointing a descriptor at the bytes we already
    /// hold (the producer / server side).
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_ADD_CLIPBOARD_TEXT,
            text: ustring_opt(self.text.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        // Producer side: build it into an owned pool, encode to raw.
        let pool = OwnedPool::new();
        let cmd = AddClipboardText::new(&pool, "réseau Frogans");
        let raw = cmd.to_raw();

        // Consumer side: decode that raw back, reading the same pooled bytes.
        let back = AddClipboardText::from_raw(raw, &pool.as_pool());
        assert_eq!(back.text.unwrap().as_str().unwrap(), "réseau Frogans");
    }

    #[test]
    fn copy_into_outlives_the_source_pool() {
        // The proxy's invariant: a command copied into our pool must stay readable
        // after the *source* pool (where it was first allocated) is freed. Build in
        // `src`, deep-copy into `dst`, drop `src`, then read from the copy.
        let dst = OwnedPool::new();
        let copy = {
            let src = OwnedPool::new();
            let cmd = AddClipboardText::new(&src, "réseau Frogans");
            cmd.copy_into(&dst)
            // `src` is dropped here — its arena bytes are gone.
        };
        assert_eq!(copy.text.unwrap().as_str().unwrap(), "réseau Frogans");
    }

    #[test]
    fn empty_text_encodes_null() {
        let cmd = AddClipboardText { text: None };
        let raw = cmd.to_raw();
        assert!(raw.text.utf8.is_null());
        assert_eq!(raw.text.len, 0);
    }
}
