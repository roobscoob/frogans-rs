//! `launch_way_out` command (engine → host) — a URL to open externally.

use fprt_sys::ui::application::launch_way_out::LaunchWayOut as Raw;
use fprt_sys::ui::application::uri_scheme::UriScheme as RawUriScheme;
use fprt_sys::ui::application::CMD_LAUNCH_WAY_OUT;
use fprt_sys::ui::{Pop, StatusName};
use fprt_sys::Fprt;

use crate::conductor::command::{Command, CommandPayload};
use crate::pool::{Pool, PooledString};

/// The URL scheme of a way-out URL. The host opens `http`/`https`/`mailto`
/// externally and errors on [`Other`](UriScheme::Other).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UriScheme {
    /// `http`.
    Http,
    /// `https`.
    Https,
    /// `mailto`.
    Mailto,
    /// Other / unknown scheme (`0x1388` or any other value).
    Other,
}

impl UriScheme {
    fn from_raw(raw: RawUriScheme) -> Self {
        match raw {
            RawUriScheme::HTTP => UriScheme::Http,
            RawUriScheme::HTTPS => UriScheme::Https,
            RawUriScheme::MAILTO => UriScheme::Mailto,
            _ => UriScheme::Other,
        }
    }
}

/// A URL the host must open externally (browser / mail client).
#[derive(Debug)]
pub struct LaunchWayOut {
    /// The URL's scheme.
    pub scheme: UriScheme,
    /// The URL.
    pub uri: Option<PooledString>,
}

impl CommandPayload for LaunchWayOut {
    const ID: StatusName = CMD_LAUNCH_WAY_OUT;
    type Raw = Raw;

    fn pop_fn(methods: &Fprt) -> Pop<Self::Raw> {
        methods.application_launch_way_out
    }

    fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `uri` was written into `pool` by the pop that produced both.
        let uri = unsafe { pool.string(raw.uri) };
        LaunchWayOut {
            scheme: UriScheme::from_raw(raw.uri_scheme),
            uri,
        }
    }

    fn into_command(self) -> Command {
        Command::ApplicationLaunchWayOut(self)
    }
}
