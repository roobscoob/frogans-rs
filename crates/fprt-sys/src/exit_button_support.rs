//! The `ExitButtonSupport` config selector.

/// Whether the application's Exit menu item is present (conductor config `+0x54`).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ExitButtonSupport(pub u32);

impl ExitButtonSupport {
    /// Exit item present (`0x2de601`, engine-side `1`).
    pub const PRESENT: ExitButtonSupport = ExitButtonSupport(0x2de601);
    /// Exit item removed (`0x2de602`, engine-side `2`).
    pub const REMOVED: ExitButtonSupport = ExitButtonSupport(0x2de602);
}
