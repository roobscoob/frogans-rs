//! `update_images` command (engine → host) — the built-in UI image set.

use fprt_sys::ui::application::update_images::UpdateImages as Raw;
use fprt_sys::ui::application::CMD_UPDATE_IMAGES;
use fprt_sys::ui::{ImageRecord, Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledImage};

/// A frame animation: the frames plus the inter-frame delay.
#[derive(Debug)]
pub struct Animation {
    /// Inter-frame delay (engine units).
    pub delay: u32,
    /// The animation frames, in order.
    pub frames: Vec<Option<PooledImage>>,
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

impl CommandPayload for UpdateImages {
    const ID: StatusName = CMD_UPDATE_IMAGES;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_update_images
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
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

    fn into_command(self) -> Command {
        Command::ApplicationUpdateImages(self)
    }
}
