//! `update_directionality` command (engine → host) — an enum, no pool.

use fprt_sys::ui::application::CMD_UPDATE_DIRECTIONALITY;
use fprt_sys::ui::application::directionality::Directionality as RawDirectionality;
use fprt_sys::ui::application::update_directionality::UpdateDirectionality as Raw;

/// Text directionality. Only [`Default`](Directionality::Default) is labelled;
/// the other two values are proven but their LTR/RTL meaning is unresolved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Directionality {
    /// `0xbb8` — default.
    Default,
    /// `0xbb9` — host selection 1 (meaning unresolved).
    Selected1,
    /// `0xbba` — host selection 2 (meaning unresolved).
    Selected2,
    /// An engine value outside the documented set.
    Other(u32),
}

impl Directionality {
    /// Map the raw enum.
    pub fn from_raw(raw: RawDirectionality) -> Self {
        match raw {
            RawDirectionality::DEFAULT => Directionality::Default,
            RawDirectionality::SELECTED_1 => Directionality::Selected1,
            RawDirectionality::SELECTED_2 => Directionality::Selected2,
            other => Directionality::Other(other.0),
        }
    }

    /// Map back to the raw enum.
    pub fn to_raw(self) -> RawDirectionality {
        match self {
            Directionality::Default => RawDirectionality::DEFAULT,
            Directionality::Selected1 => RawDirectionality::SELECTED_1,
            Directionality::Selected2 => RawDirectionality::SELECTED_2,
            Directionality::Other(v) => RawDirectionality(v),
        }
    }
}

/// The text directionality the host should apply.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UpdateDirectionality {
    /// The directionality enum.
    pub directionality: Directionality,
}

impl UpdateDirectionality {
    /// Build one (no pool — enum payload).
    pub fn new(directionality: Directionality) -> Self {
        UpdateDirectionality { directionality }
    }

    /// Decode the engine's payload.
    pub fn from_raw(raw: Raw) -> Self {
        UpdateDirectionality {
            directionality: Directionality::from_raw(raw.directionality),
        }
    }

    /// Encode into the raw payload.
    pub fn to_raw(&self) -> Raw {
        Raw {
            status_id: CMD_UPDATE_DIRECTIONALITY,
            directionality: self.directionality.to_raw(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_every_variant() {
        for d in [
            Directionality::Default,
            Directionality::Selected1,
            Directionality::Selected2,
            Directionality::Other(0x1234),
        ] {
            let p = UpdateDirectionality::new(d);
            assert_eq!(UpdateDirectionality::from_raw(p.to_raw()), p);
        }
    }
}
