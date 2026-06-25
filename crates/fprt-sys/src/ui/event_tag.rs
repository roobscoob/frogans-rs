//! The `EventTag` event id (field 0 of every event payload).

/// The `0x10ccxx` external event id the host writes into an event payload's
/// field 0; the engine validates it and hard-rejects a mismatch. The specific
/// tags live with their component.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EventTag(pub u32);
