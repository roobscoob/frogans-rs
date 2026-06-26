//! Safe configuration for [`Library::spawn_conductor`](crate::Library::spawn_conductor).

use std::borrow::Cow;
use std::path::Path;

use fprt_sys::deployment_mode::DeploymentMode as RawDeploymentMode;
use fprt_sys::devtools_support::DevtoolsSupport;
use fprt_sys::exit_button_support::ExitButtonSupport;
use fprt_sys::image_format::ImageFormat as RawImageFormat;
use fprt_sys::nature::Nature as RawNature;
use fprt_sys::reserved_flag::ReservedFlag;
use fprt_sys::start_information::StartInformation;

use crate::call::ustring;

/// The Frogans application's nature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nature {
    /// A public application.
    Public,
    /// An experimental application.
    Experimental,
}

impl Nature {
    fn raw(self) -> RawNature {
        match self {
            Nature::Public => RawNature::PUBLIC,
            Nature::Experimental => RawNature::EXPERIMENTAL,
        }
    }
}

/// Pixel format for images the engine encodes and hands the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG-encoded.
    Png,
    /// Raw RGBA, straight alpha.
    Rgba,
    /// Raw ABGR, straight alpha.
    Abgr,
    /// Raw ARGB, straight alpha.
    Argb,
    /// Raw BGRA, straight alpha.
    Bgra,
    /// Raw RGBA, premultiplied alpha.
    RgbaPremultiplied,
    /// Raw ABGR, premultiplied alpha.
    AbgrPremultiplied,
    /// Raw ARGB, premultiplied alpha.
    ArgbPremultiplied,
    /// Raw BGRA, premultiplied alpha.
    BgraPremultiplied,
}

impl ImageFormat {
    /// The format the engine ships by default for the platform — BGRA-premultiplied
    /// on Windows, RGBA-premultiplied on macOS.
    pub fn platform_default() -> Self {
        #[cfg(windows)]
        {
            ImageFormat::BgraPremultiplied
        }
        #[cfg(not(windows))]
        {
            ImageFormat::RgbaPremultiplied
        }
    }

    fn raw(self) -> RawImageFormat {
        match self {
            ImageFormat::Png => RawImageFormat::PNG,
            ImageFormat::Rgba => RawImageFormat::RGBA,
            ImageFormat::Abgr => RawImageFormat::ABGR,
            ImageFormat::Argb => RawImageFormat::ARGB,
            ImageFormat::Bgra => RawImageFormat::BGRA,
            ImageFormat::RgbaPremultiplied => RawImageFormat::RGBA_PREMULTIPLIED,
            ImageFormat::AbgrPremultiplied => RawImageFormat::ABGR_PREMULTIPLIED,
            ImageFormat::ArgbPremultiplied => RawImageFormat::ARGB_PREMULTIPLIED,
            ImageFormat::BgraPremultiplied => RawImageFormat::BGRA_PREMULTIPLIED,
        }
    }
}

/// Deployment mode. The distinction between the two is **not proven** — there is
/// no consumer in the engine binary to confirm a production-vs-test meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeploymentMode {
    /// Engine-side `201` (meaning unproven).
    First,
    /// Engine-side `202` (meaning unproven).
    Second,
}

impl DeploymentMode {
    fn raw(self) -> RawDeploymentMode {
        match self {
            DeploymentMode::First => RawDeploymentMode::FIRST,
            DeploymentMode::Second => RawDeploymentMode::SECOND,
        }
    }
}

/// The four writable server-root directories the engine needs.
///
/// Paths must be UTF-8. [`new`](Self::new) borrows them for the spawn call only;
/// [`under`](Self::under) derives and owns them.
#[derive(Clone)]
pub struct Directories<'a> {
    user_data: Cow<'a, str>,
    resources: Cow<'a, str>,
    developers: Cow<'a, str>,
    developers_test: Cow<'a, str>,
}

impl<'a> Directories<'a> {
    /// The four directories, in order: user-data, resources (holds the engine
    /// fonts), developers, developers-test.
    pub fn new(
        user_data: &'a str,
        resources: &'a str,
        developers: &'a str,
        developers_test: &'a str,
    ) -> Self {
        Directories {
            user_data: Cow::Borrowed(user_data),
            resources: Cow::Borrowed(resources),
            developers: Cow::Borrowed(developers),
            developers_test: Cow::Borrowed(developers_test),
        }
    }

    /// Derive the four as subdirectories of `base`: `base/user_data`,
    /// `base/resources`, `base/developers`, `base/developers_test`.
    ///
    /// (`in` is a reserved keyword, so this isn't named `in`.) A non-UTF-8 `base`
    /// is lossily converted.
    pub fn under(base: impl AsRef<Path>) -> Directories<'static> {
        let base = base.as_ref();
        let sub = |name: &str| Cow::Owned(base.join(name).to_string_lossy().into_owned());
        Directories {
            user_data: sub("user_data"),
            resources: sub("resources"),
            developers: sub("developers"),
            developers_test: sub("developers_test"),
        }
    }
}

/// The Frogans application's manifest identity.
#[derive(Clone, Copy)]
pub struct Manifest<'a> {
    target_id: &'a str,
    channel_id: &'a str,
    originator_id: &'a str,
    comment: &'a str,
    version: (u32, u32, u32),
}

impl<'a> Manifest<'a> {
    /// The three required manifest ids. Comment defaults to empty, version to
    /// `0.0.0` — set them with [`with_comment`](Self::with_comment) /
    /// [`with_version`](Self::with_version).
    pub fn new(target_id: &'a str, channel_id: &'a str, originator_id: &'a str) -> Self {
        Manifest {
            target_id,
            channel_id,
            originator_id,
            comment: "",
            version: (0, 0, 0),
        }
    }

    /// Set the manifest comment.
    pub fn with_comment(mut self, comment: &'a str) -> Self {
        self.comment = comment;
        self
    }

    /// Set the manifest version (`major.minor.patch`).
    pub fn with_version(mut self, major: u32, minor: u32, patch: u32) -> Self {
        self.version = (major, minor, patch);
        self
    }
}

/// Configuration for [`Library::spawn_conductor`](crate::Library::spawn_conductor).
///
/// Built from the required [`Directories`] + [`Manifest`], then chained setters
/// for the optional selectors (default: platform image format, [`Nature::Public`],
/// devtools off, exit button present). All strings are borrowed for the spawn
/// call only — the engine reads them synchronously and does not retain them.
///
/// ```ignore
/// let manifest = Manifest::new(target, channel, originator).with_version(0, 6, 1);
/// let directories = Directories::new(&data, &res, &dev, &dev_test);
/// let conductor = library.spawn_conductor(
///     ConductorConfig::new(directories, manifest).devtools(true),
/// )?;
/// ```
pub struct ConductorConfig<'a> {
    directories: Directories<'a>,
    manifest: Manifest<'a>,
    nature: Nature,
    devtools: bool,
    exit_button: bool,
    standalone_image_format: ImageFormat,
    site_image_format: ImageFormat,
    deployment_mode: DeploymentMode,
}

impl<'a> ConductorConfig<'a> {
    /// A config from the required directories + manifest, with default selectors.
    pub fn new(directories: Directories<'a>, manifest: Manifest<'a>) -> Self {
        ConductorConfig {
            directories,
            manifest,
            nature: Nature::Public,
            devtools: false,
            exit_button: true,
            standalone_image_format: ImageFormat::platform_default(),
            site_image_format: ImageFormat::platform_default(),
            deployment_mode: DeploymentMode::First,
        }
    }

    /// The application's nature (default [`Nature::Public`]).
    pub fn nature(mut self, nature: Nature) -> Self {
        self.nature = nature;
        self
    }

    /// Pixel format for standalone images (default [`ImageFormat::platform_default`]).
    pub fn standalone_image_format(mut self, format: ImageFormat) -> Self {
        self.standalone_image_format = format;
        self
    }

    /// Pixel format for xrepresentation / site images
    /// (default [`ImageFormat::platform_default`]).
    pub fn site_image_format(mut self, format: ImageFormat) -> Self {
        self.site_image_format = format;
        self
    }

    /// The deployment mode (default [`DeploymentMode::First`]).
    pub fn deployment_mode(mut self, mode: DeploymentMode) -> Self {
        self.deployment_mode = mode;
        self
    }

    /// Whether the developer-tools UI is available (default `false`).
    pub fn devtools(mut self, enabled: bool) -> Self {
        self.devtools = enabled;
        self
    }

    /// Whether the Exit menu item is present (default `true`).
    pub fn exit_button(mut self, present: bool) -> Self {
        self.exit_button = present;
        self
    }

    /// Materialize the raw `StartInformation`. The `Ustring`s borrow `self`'s
    /// strings, so the returned value must not outlive `self`.
    pub(crate) fn to_raw(&self) -> StartInformation {
        let d = &self.directories;
        let m = &self.manifest;
        StartInformation {
            user_data: ustring(&d.user_data),
            fonts: ustring(&d.resources),
            developers: ustring(&d.developers),
            developers_test: ustring(&d.developers_test),
            imgfmt_a: self.standalone_image_format.raw(),
            imgfmt_b: self.site_image_format.raw(),
            reserved_flag: ReservedFlag::SECOND,
            deployment_mode: self.deployment_mode.raw(),
            devtools_support: if self.devtools {
                DevtoolsSupport::ENABLED
            } else {
                DevtoolsSupport::DISABLED
            },
            exit_button_support: if self.exit_button {
                ExitButtonSupport::PRESENT
            } else {
                ExitButtonSupport::REMOVED
            },
            nature: self.nature.raw(),
            manifest_target_id: ustring(m.target_id),
            manifest_channel_id: ustring(m.channel_id),
            manifest_originator_id: ustring(m.originator_id),
            manifest_ver_major: m.version.0,
            manifest_ver_minor: m.version.1,
            manifest_ver_patch: m.version.2,
            manifest_comment: ustring(m.comment),
        }
    }
}
