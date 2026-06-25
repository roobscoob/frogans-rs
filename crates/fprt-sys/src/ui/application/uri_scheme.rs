//! The `UriScheme` selector (`command_launch_way_out`).

/// URL scheme for a way-out URL. The host opens `http`/`https`/`mailto`
/// externally and errors on [`UriScheme::OTHER`].
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UriScheme(pub u32);

impl UriScheme {
    /// `0x1388` (5000) — other / unknown scheme.
    pub const OTHER: UriScheme = UriScheme(0x1388);
    /// `0x1389` (5001) — `http`.
    pub const HTTP: UriScheme = UriScheme(0x1389);
    /// `0x138a` (5002) — `https`.
    pub const HTTPS: UriScheme = UriScheme(0x138a);
    /// `0x138b` (5003) — `mailto`.
    pub const MAILTO: UriScheme = UriScheme(0x138b);
}
