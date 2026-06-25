//! `command_update_images` payload (`0x228`).
//!
//! The engine's complete built-in UI image set, handed to the host once at
//! application start. Offsets are made explicit with reserved fields: the
//! engine memsets and writes the full `0x228` span, and several gaps are NOT
//! alignment padding, so they must be declared to land each field correctly.

use crate::ui::ImageRecord;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UpdateImages {
    /// +0x000  field 0 — engine-stamped status name (`0x2195aa` on success).
    pub type_tag: u32,
    pub _rsv004: u32,
    /// +0x008  normal-mode pad icon.
    pub pad_main: ImageRecord,
    pub _rsv020: u64,
    /// +0x028  pad animation delay.
    pub pad_anim_delay: u32,
    pub _rsv02c: u32,
    /// +0x030  pad animation frame count.
    pub pad_anim_count: i32,
    pub _rsv034: u32,
    /// +0x038  base of the Pad's animation frames — `pad_anim_count` `ImageRecord`s,
    /// played at `pad_anim_delay` (the Pad's working/loading animation; role inferred).
    pub pad_anim_images: *mut ImageRecord,
    /// +0x040  discreet-mode pad icon.
    pub pad_main_discreet: ImageRecord,
    pub _rsv058: u64,
    /// +0x060  site animation delay.
    pub site_anim_delay: u32,
    pub _rsv064: u32,
    /// +0x068  site animation frame count.
    pub site_anim_count: i32,
    pub _rsv06c: u32,
    /// +0x070  base of the site-loading animation frames — `site_anim_count`
    /// `ImageRecord`s, played at `site_anim_delay`.
    pub site_anim_images: *mut ImageRecord,
    /// +0x078  tooltip images, ids 0..15 (8 element types × hover / selected).
    pub tooltip: [ImageRecord; 16],
    /// +0x1f8  ring at rest.
    pub ring_released: ImageRecord,
    /// +0x210  ring grabbed.
    pub ring_captured: ImageRecord,
}
