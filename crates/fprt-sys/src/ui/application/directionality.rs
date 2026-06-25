//! The `Directionality` selector (`command_update_directionality`).

/// Text directionality enum. Only [`Directionality::DEFAULT`] is labelled; the
/// other two values are proven but their LTR/RTL meaning is **[unresolved]**.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Directionality(pub u32);

impl Directionality {
    /// `0xbb8` (3000) — default.
    pub const DEFAULT: Directionality = Directionality(0xbb8);
    /// `0xbb9` (3001) — host selection 1; meaning unresolved.
    pub const SELECTED_1: Directionality = Directionality(0xbb9);
    /// `0xbba` (3002) — host selection 2; meaning unresolved.
    pub const SELECTED_2: Directionality = Directionality(0xbba);
}
