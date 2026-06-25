//! The `DeploymentMode` config selector.

/// Deployment-mode marker in the conductor config (`+0x4c`). Validated and
/// stored; the wrapper remaps it to a small int (201/202) for the engine.
///
/// The distinction between the two values is **not proven** — there is no
/// consumer in the binary to confirm a production-vs-test meaning.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeploymentMode(pub u32);

impl DeploymentMode {
    /// `0x2dda49` (engine-side `201`). Meaning unproven.
    pub const FIRST: DeploymentMode = DeploymentMode(0x2dda49);
    /// `0x2dda4a` (engine-side `202`). Meaning unproven.
    pub const SECOND: DeploymentMode = DeploymentMode(0x2dda4a);
}
