//! The `DevtoolsSupport` config selector.

/// Whether the developer-tools UI is available (conductor config `+0x50`).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DevtoolsSupport(pub u32);

impl DevtoolsSupport {
    /// Devtools enabled (`0x2dde31`, engine-side `1`).
    pub const ENABLED: DevtoolsSupport = DevtoolsSupport(0x2dde31);
    /// Devtools disabled (`0x2dde32`, engine-side `2`).
    pub const DISABLED: DevtoolsSupport = DevtoolsSupport(0x2dde32);
}
