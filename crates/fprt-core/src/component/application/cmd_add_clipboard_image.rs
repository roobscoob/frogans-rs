//! `add_clipboard_image` command (engine → host) — a **pooled** image.

use fprt_sys::ui::application::CMD_ADD_CLIPBOARD_IMAGE;
use fprt_sys::ui::application::add_clipboard_image::AddClipboardImage as Raw;

use crate::pool::{OwnedPool, Pool, PooledImage};
use crate::wire::image_record;

/// An image the host must place on the system clipboard.
#[derive(Debug)]
pub struct AddClipboardImage {
    /// The clipboard image.
    pub image: Option<PooledImage>,
}

impl AddClipboardImage {
    /// Build one, allocating the encoded image into `pool`.
    pub fn new(pool: &OwnedPool, bytes: &[u8], width: u32, height: u32) -> Self {
        AddClipboardImage {
            image: Some(pool.alloc_image(bytes, width, height)),
        }
    }

    /// Decode the engine's payload, wrapping the pooled image zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.image` was written into `pool` by the pop that produced both.
        AddClipboardImage {
            image: unsafe { pool.image(raw.image) },
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        AddClipboardImage {
            image: pool.clone_image_opt(&self.image),
        }
    }

    /// Encode into the raw payload, pointing a descriptor at the bytes we hold.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_ADD_CLIPBOARD_IMAGE,
            image: image_record(self.image.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let pixels = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let cmd = AddClipboardImage::new(&pool, &pixels, 2, 1);
        let raw = cmd.to_raw();
        let back = AddClipboardImage::from_raw(raw, &pool.as_pool());
        let img = back.image.unwrap();
        assert_eq!(img.bytes(), &pixels);
        assert_eq!((img.width(), img.height()), (2, 1));
    }

    #[test]
    fn empty_image_encodes_null() {
        let raw = AddClipboardImage { image: None }.to_raw();
        assert!(raw.image.buffer.is_null());
        assert_eq!(raw.image.byte_len, 0);
    }
}
