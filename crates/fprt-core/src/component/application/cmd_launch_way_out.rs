//! `launch_way_out` command (engine → host) — an enum + a **pooled** string.

use fprt_sys::ui::StatusName;
use fprt_sys::ui::application::launch_way_out::LaunchWayOut as Raw;
use fprt_sys::ui::application::uri_scheme::UriScheme as RawUriScheme;

use crate::pool::{OwnedPool, Pool, PooledString};
use crate::wire::ustring_opt;

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
    /// Other / unknown scheme.
    Other,
}

impl UriScheme {
    /// Map the raw enum (any unknown value ⇒ [`Other`](UriScheme::Other)).
    pub fn from_raw(raw: RawUriScheme) -> Self {
        match raw {
            RawUriScheme::HTTP => UriScheme::Http,
            RawUriScheme::HTTPS => UriScheme::Https,
            RawUriScheme::MAILTO => UriScheme::Mailto,
            _ => UriScheme::Other,
        }
    }

    /// Map back to the raw enum ([`Other`](UriScheme::Other) ⇒ the `OTHER` sentinel).
    pub fn to_raw(self) -> RawUriScheme {
        match self {
            UriScheme::Http => RawUriScheme::HTTP,
            UriScheme::Https => RawUriScheme::HTTPS,
            UriScheme::Mailto => RawUriScheme::MAILTO,
            UriScheme::Other => RawUriScheme::OTHER,
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

impl LaunchWayOut {
    /// Build one, allocating `uri` into `pool`.
    pub fn new(pool: &OwnedPool, scheme: UriScheme, uri: &str) -> Self {
        LaunchWayOut {
            scheme,
            uri: Some(pool.alloc_str(uri)),
        }
    }

    /// Decode the engine's payload, wrapping the pooled URL zero-copy.
    pub fn from_raw(raw: Raw, pool: &Pool) -> Self {
        // SAFETY: `raw.uri` was written into `pool` by the pop that produced both.
        LaunchWayOut {
            scheme: UriScheme::from_raw(raw.uri_scheme),
            uri: unsafe { pool.string(raw.uri) },
        }
    }

    /// Deep-copy into `pool` — copies the bytes, so the result borrows no other pool.
    pub fn copy_into(&self, pool: &OwnedPool) -> Self {
        LaunchWayOut {
            scheme: self.scheme,
            uri: pool.clone_str_opt(&self.uri),
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            // Field 0 is `NONE` when empty, `FALLBACK` when a URL is present.
            type_tag: if self.uri.is_some() {
                StatusName::FALLBACK
            } else {
                StatusName::NONE
            },
            uri_scheme: self.scheme.to_raw(),
            uri: ustring_opt(self.uri.as_ref()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_pool() {
        let pool = OwnedPool::new();
        let cmd = LaunchWayOut::new(&pool, UriScheme::Https, "https://frogans.example/");
        let raw = cmd.to_raw();
        let back = LaunchWayOut::from_raw(raw, &pool.as_pool());
        assert_eq!(back.scheme, UriScheme::Https);
        assert_eq!(back.uri.unwrap().as_str().unwrap(), "https://frogans.example/");
    }
}
