//! The `StartInformation` conductor-boot config.

use crate::deployment_mode::DeploymentMode;
use crate::devtools_support::DevtoolsSupport;
use crate::exit_button_support::ExitButtonSupport;
use crate::image_format::ImageFormat;
use crate::nature::Nature;
use crate::reserved_flag::ReservedFlag;
use crate::ustring::Ustring;

/// The wire config passed to `fprt_conductor_start` (`0xb0` bytes, same layout
/// on both platforms).
///
/// The two implicit 4-byte gaps (after `nature` and after `manifest_ver_patch`)
/// are `repr(C)` alignment padding that keep the following `Ustring` 8-byte
/// aligned — these are the original's `_pad5c` / `_pad9c`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StartInformation {
    // Four writable server-root directories (`+0x00..0x40`).
    /// User-data directory (`+0x00`).
    pub user_data: Ustring,
    /// Resources directory (`+0x10`); holds the fonts (`fsdl-fonts.dat`).
    pub fonts: Ustring,
    /// Developers directory (`+0x20`).
    pub developers: Ustring,
    /// Developers-test directory (`+0x30`).
    pub developers_test: Ustring,
    /// Standalone-image pixel format (`+0x40`).
    pub imgfmt_a: ImageFormat,
    /// Xrepresentation / site pixel format (`+0x44`).
    pub imgfmt_b: ImageFormat,
    /// Reserved (`+0x48`).
    pub reserved_flag: ReservedFlag,
    /// Deployment mode (`+0x4c`).
    pub deployment_mode: DeploymentMode,
    /// Developer-tools availability (`+0x50`).
    pub devtools_support: DevtoolsSupport,
    /// Exit-menu-item availability (`+0x54`).
    pub exit_button_support: ExitButtonSupport,
    /// Application nature (`+0x58`).
    pub nature: Nature,
    // +0x5c: 4 bytes implicit padding → manifest_target_id aligns to +0x60.
    /// Manifest target id (`+0x60`).
    pub manifest_target_id: Ustring,
    /// Manifest channel id (`+0x70`).
    pub manifest_channel_id: Ustring,
    /// Manifest originator id (`+0x80`).
    pub manifest_originator_id: Ustring,
    /// Manifest version, major (`+0x90`).
    pub manifest_ver_major: u32,
    /// Manifest version, minor (`+0x94`).
    pub manifest_ver_minor: u32,
    /// Manifest version, patch (`+0x98`).
    pub manifest_ver_patch: u32,
    // +0x9c: 4 bytes implicit padding → manifest_comment aligns to +0xa0.
    /// Manifest comment (`+0xa0..0xb0`).
    pub manifest_comment: Ustring,
}
