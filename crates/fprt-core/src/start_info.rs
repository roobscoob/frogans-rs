//! [`StartInfo`] — the borrowed, safe view of a conductor's [`StartInformation`]
//! boot config.
//!
//! The host hands `conductor_start` a `*const StartInformation` (four server-root
//! paths, the pixel-format + mode enums, and the manifest identity). Like an event,
//! it is *borrowed* for the call: the server decodes it (`from_raw`) to validate
//! and use; the client encodes it (`to_raw`) to make the call. Strings view the
//! host buffer for the call's duration, so the type carries a lifetime.

use fprt_sys::deployment_mode::DeploymentMode;
use fprt_sys::devtools_support::DevtoolsSupport;
use fprt_sys::exit_button_support::ExitButtonSupport;
use fprt_sys::image_format::ImageFormat;
use fprt_sys::nature::Nature;
use fprt_sys::reserved_flag::ReservedFlag;
use fprt_sys::start_information::StartInformation;

use crate::wire::{as_str, ustring};

/// A conductor's boot config, decoded for safe inspection. Borrowed for the
/// `conductor_start` call only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartInfo<'a> {
    /// User-data directory (writable).
    pub user_data: &'a str,
    /// Resources directory holding the fonts (`fsdl-fonts.dat`).
    pub fonts: &'a str,
    /// Developers directory.
    pub developers: &'a str,
    /// Developers-test directory.
    pub developers_test: &'a str,
    /// Standalone-image pixel format.
    pub imgfmt_a: ImageFormat,
    /// Xrepresentation / site pixel format.
    pub imgfmt_b: ImageFormat,
    /// Reserved flag.
    pub reserved_flag: ReservedFlag,
    /// Deployment mode.
    pub deployment_mode: DeploymentMode,
    /// Developer-tools availability.
    pub devtools_support: DevtoolsSupport,
    /// Exit-menu-item availability.
    pub exit_button_support: ExitButtonSupport,
    /// Application nature.
    pub nature: Nature,
    /// Manifest target id.
    pub manifest_target_id: &'a str,
    /// Manifest channel id.
    pub manifest_channel_id: &'a str,
    /// Manifest originator id.
    pub manifest_originator_id: &'a str,
    /// Manifest version, major.
    pub manifest_ver_major: u32,
    /// Manifest version, minor.
    pub manifest_ver_minor: u32,
    /// Manifest version, patch.
    pub manifest_ver_patch: u32,
    /// Manifest comment.
    pub manifest_comment: &'a str,
}

impl StartInfo<'static> {
    /// An all-empty config — every path/id blank, every enum zeroed. For tests and
    /// trivial engines that ignore the config; a validating engine would reject it.
    pub const EMPTY: StartInfo<'static> = StartInfo {
        user_data: "",
        fonts: "",
        developers: "",
        developers_test: "",
        imgfmt_a: ImageFormat(0),
        imgfmt_b: ImageFormat(0),
        reserved_flag: ReservedFlag(0),
        deployment_mode: DeploymentMode(0),
        devtools_support: DevtoolsSupport(0),
        exit_button_support: ExitButtonSupport(0),
        nature: Nature(0),
        manifest_target_id: "",
        manifest_channel_id: "",
        manifest_originator_id: "",
        manifest_ver_major: 0,
        manifest_ver_minor: 0,
        manifest_ver_patch: 0,
        manifest_comment: "",
    };
}

impl<'a> StartInfo<'a> {
    /// Whether the developer-tools UI is enabled (decodes [`Self::devtools_support`]
    /// to a `bool` — the form the client's `ConductorConfig` takes).
    pub fn devtools_enabled(&self) -> bool {
        self.devtools_support == DevtoolsSupport::ENABLED
    }

    /// Whether the Exit menu item is present (decodes [`Self::exit_button_support`]).
    pub fn exit_button_present(&self) -> bool {
        self.exit_button_support == ExitButtonSupport::PRESENT
    }

    /// Decode the inbound config, borrowing its strings for the call (the server /
    /// consumer side).
    pub fn from_raw(raw: &'a StartInformation) -> Self {
        // SAFETY: each `Ustring` field is valid for the `conductor_start` call's
        // duration (host contract), so the borrowed `&str`s live as long as `raw`.
        unsafe {
            StartInfo {
                user_data: as_str(raw.user_data),
                fonts: as_str(raw.fonts),
                developers: as_str(raw.developers),
                developers_test: as_str(raw.developers_test),
                imgfmt_a: raw.imgfmt_a,
                imgfmt_b: raw.imgfmt_b,
                reserved_flag: raw.reserved_flag,
                deployment_mode: raw.deployment_mode,
                devtools_support: raw.devtools_support,
                exit_button_support: raw.exit_button_support,
                nature: raw.nature,
                manifest_target_id: as_str(raw.manifest_target_id),
                manifest_channel_id: as_str(raw.manifest_channel_id),
                manifest_originator_id: as_str(raw.manifest_originator_id),
                manifest_ver_major: raw.manifest_ver_major,
                manifest_ver_minor: raw.manifest_ver_minor,
                manifest_ver_patch: raw.manifest_ver_patch,
                manifest_comment: as_str(raw.manifest_comment),
            }
        }
    }

    /// Encode into the raw config, pointing each descriptor at the strings we hold
    /// (the client / producer side). The strings must outlive the call.
    pub fn to_raw(&self) -> StartInformation {
        StartInformation {
            user_data: ustring(self.user_data),
            fonts: ustring(self.fonts),
            developers: ustring(self.developers),
            developers_test: ustring(self.developers_test),
            imgfmt_a: self.imgfmt_a,
            imgfmt_b: self.imgfmt_b,
            reserved_flag: self.reserved_flag,
            deployment_mode: self.deployment_mode,
            devtools_support: self.devtools_support,
            exit_button_support: self.exit_button_support,
            nature: self.nature,
            manifest_target_id: ustring(self.manifest_target_id),
            manifest_channel_id: ustring(self.manifest_channel_id),
            manifest_originator_id: ustring(self.manifest_originator_id),
            manifest_ver_major: self.manifest_ver_major,
            manifest_ver_minor: self.manifest_ver_minor,
            manifest_ver_patch: self.manifest_ver_patch,
            manifest_comment: ustring(self.manifest_comment),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let info = StartInfo {
            user_data: "/data",
            fonts: "/res",
            developers: "/dev",
            developers_test: "/dev-test",
            imgfmt_a: ImageFormat::BGRA_PREMULTIPLIED,
            imgfmt_b: ImageFormat::PNG,
            reserved_flag: ReservedFlag(0),
            deployment_mode: DeploymentMode(0),
            devtools_support: DevtoolsSupport(0),
            exit_button_support: ExitButtonSupport(0),
            nature: Nature::PUBLIC,
            manifest_target_id: "target",
            manifest_channel_id: "channel",
            manifest_originator_id: "origin",
            manifest_ver_major: 1,
            manifest_ver_minor: 2,
            manifest_ver_patch: 3,
            manifest_comment: "a comment",
        };
        let raw = info.to_raw();
        assert_eq!(StartInfo::from_raw(&raw), info);
    }
}
