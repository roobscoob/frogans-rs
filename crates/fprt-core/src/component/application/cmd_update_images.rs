//! `update_images` command (engine → host) — the built-in UI image set.
//!
//! Complex multi-image payload, encoded both ways: the client decodes the
//! engine's set once at start ([`from_raw`](UpdateImages::from_raw)); the server
//! builds one into a pool and encodes it ([`to_raw`](UpdateImages::to_raw)).

use fprt_sys::ui::ImageRecord;
use fprt_sys::ui::application::CMD_UPDATE_IMAGES;
use fprt_sys::ui::application::update_images::UpdateImages as Raw;

use crate::pool::{OwnedPool, Pool, PooledImage};
use crate::wire::image_record;

/// A frame animation: the frames plus the inter-frame delay.
#[derive(Debug)]
pub struct Animation {
    /// Inter-frame delay (engine units).
    pub delay: u32,
    /// The animation frames, in order.
    pub frames: Vec<Option<PooledImage>>,
}

impl Animation {
    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        Animation {
            delay: self.delay,
            frames: self.frames.iter().map(|i| pool.clone_image_opt(i)).collect(),
        }
    }
}

/// The engine's complete built-in UI image set, handed to the host once at
/// application start.
#[derive(Debug)]
pub struct UpdateImages {
    /// Normal-mode pad icon.
    pub pad_main: Option<PooledImage>,
    /// Discreet-mode pad icon.
    pub pad_main_discreet: Option<PooledImage>,
    /// The pad's working/loading animation.
    pub pad_animation: Animation,
    /// The site-loading animation.
    pub site_animation: Animation,
    /// Tooltip images, ids 0..15 (8 element types × hover / selected).
    pub tooltips: [Option<PooledImage>; 16],
    /// Ring at rest.
    pub ring_released: Option<PooledImage>,
    /// Ring grabbed.
    pub ring_captured: Option<PooledImage>,
}

/// Walk a `*mut ImageRecord` array of `count` entries into pooled images.
///
/// # Safety
/// `ptr` must point at `count` records in `pool` (or be null).
unsafe fn images(ptr: *mut ImageRecord, count: i32, pool: &Pool) -> Vec<Option<PooledImage>> {
    let n = count.max(0) as usize;
    let mut out = Vec::with_capacity(n);
    if !ptr.is_null() {
        for i in 0..n {
            out.push(unsafe { pool.image(*ptr.add(i)) });
        }
    }
    out
}

impl UpdateImages {
    /// Decode the engine's payload, wrapping every image zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: every image record (scalars, the two frame arrays, and the
        // tooltip array) points into `pool`, the pool that produced this pop.
        unsafe {
            UpdateImages {
                pad_main: pool.image(raw.pad_main),
                pad_main_discreet: pool.image(raw.pad_main_discreet),
                pad_animation: Animation {
                    delay: raw.pad_anim_delay,
                    frames: images(raw.pad_anim_images, raw.pad_anim_count, pool),
                },
                site_animation: Animation {
                    delay: raw.site_anim_delay,
                    frames: images(raw.site_anim_images, raw.site_anim_count, pool),
                },
                tooltips: core::array::from_fn(|i| pool.image(raw.tooltip[i])),
                ring_released: pool.image(raw.ring_released),
                ring_captured: pool.image(raw.ring_captured),
            }
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        UpdateImages {
            pad_main: pool.clone_image_opt(&self.pad_main),
            pad_main_discreet: pool.clone_image_opt(&self.pad_main_discreet),
            pad_animation: self.pad_animation.copy_into(pool),
            site_animation: self.site_animation.copy_into(pool),
            tooltips: core::array::from_fn(|i| pool.clone_image_opt(&self.tooltips[i])),
            ring_released: pool.clone_image_opt(&self.ring_released),
            ring_captured: pool.clone_image_opt(&self.ring_captured),
        }
    }

    /// Build a small representative set into `pool` — a scalar pad icon, a ring,
    /// a 2-frame pad animation, and one populated tooltip slot. Just enough to
    /// exercise every encode path (a scalar, a non-empty frame array, a populated
    /// and an empty tooltip slot); the engine ships a far fuller set.
    pub fn new(pool: &OwnedPool) -> Self {
        UpdateImages {
            pad_main: Some(pool.alloc_image(&[0xff], 1, 1)),
            pad_main_discreet: None,
            pad_animation: Animation {
                delay: 80,
                frames: vec![
                    Some(pool.alloc_image(&[0x01], 1, 1)),
                    Some(pool.alloc_image(&[0x02], 1, 1)),
                ],
            },
            site_animation: Animation {
                delay: 0,
                frames: Vec::new(),
            },
            tooltips: core::array::from_fn(|i| {
                (i == 3).then(|| pool.alloc_image(&[0x07, 0x08], 2, 1))
            }),
            ring_released: Some(pool.alloc_image(&[0xaa], 1, 1)),
            ring_captured: None,
        }
    }

    /// Encode into the raw payload, allocating the two frame arrays into `pool`.
    pub fn to_raw(&self, pool: &OwnedPool) -> Raw {
        // Build a frame array into the pool, yielding its `(count, base)` — null
        // and 0 for an empty animation.
        let frames = |frames: &[Option<PooledImage>]| -> (i32, *mut ImageRecord) {
            if frames.is_empty() {
                (0, core::ptr::null_mut())
            } else {
                let records: Vec<ImageRecord> =
                    frames.iter().map(|f| image_record(f.as_ref())).collect();
                (
                    records.len() as i32,
                    pool.alloc_slice(&records).cast::<ImageRecord>() as *mut ImageRecord,
                )
            }
        };
        let (pad_anim_count, pad_anim_images) = frames(&self.pad_animation.frames);
        let (site_anim_count, site_anim_images) = frames(&self.site_animation.frames);
        Raw {
            type_tag: CMD_UPDATE_IMAGES.0,
            _rsv004: 0,
            pad_main: image_record(self.pad_main.as_ref()),
            _rsv020: 0,
            pad_anim_delay: self.pad_animation.delay,
            _rsv02c: 0,
            pad_anim_count,
            _rsv034: 0,
            pad_anim_images,
            pad_main_discreet: image_record(self.pad_main_discreet.as_ref()),
            _rsv058: 0,
            site_anim_delay: self.site_animation.delay,
            _rsv064: 0,
            site_anim_count,
            _rsv06c: 0,
            site_anim_images,
            tooltip: core::array::from_fn(|i| image_record(self.tooltips[i].as_ref())),
            ring_released: image_record(self.ring_released.as_ref()),
            ring_captured: image_record(self.ring_captured.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let back = UpdateImages::from_raw(UpdateImages::new(&pool).to_raw(&pool), &pool.as_pool());

        // Scalar images: the populated ones decode, the `None` ones stay absent.
        let pad = back.pad_main.expect("pad_main present");
        assert_eq!(pad.bytes(), &[0xff]);
        assert_eq!((pad.width(), pad.height()), (1, 1));
        assert!(back.pad_main_discreet.is_none());
        assert!(back.ring_released.is_some());
        assert!(back.ring_captured.is_none());

        // Animations: counts, delays, and frame bytes survive; empty stays empty.
        assert_eq!(back.pad_animation.delay, 80);
        assert_eq!(back.pad_animation.frames.len(), 2);
        let f0 = back.pad_animation.frames[0].as_ref().expect("frame 0 present");
        assert_eq!(f0.bytes(), &[0x01]);
        assert!(back.site_animation.frames.is_empty());

        // Tooltips: only slot 3 is populated.
        let tip = back.tooltips[3].as_ref().expect("tooltip 3 present");
        assert_eq!(tip.bytes(), &[0x07, 0x08]);
        assert_eq!((tip.width(), tip.height()), (2, 1));
        assert!(back.tooltips[0].is_none());
        assert!(back.tooltips[15].is_none());
    }
}
