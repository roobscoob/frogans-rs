//! `leaptofrogans` event (host → engine) — a **borrowed** payload.
//!
//! Reference event codec: one borrowed string. No pool — the data need only
//! outlive the call. Construct it (`new`) by holding a borrow, encode it
//! (`to_raw`) by pointing a descriptor at that borrow, and decode it (`from_raw`)
//! by borrowing the inbound payload's bytes for the call.

use fprt_sys::ui::application::EVT_LEAPTOFROGANS;
use fprt_sys::ui::application::event_leaptofrogans::EventLeaptofrogans as Raw;

use crate::wire::{as_str, ustring};

/// A request to open a Frogans address (the pad is visible).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportLeaptofrogans<'a> {
    /// The Frogans address to open (borrowed for the call only).
    pub address: &'a str,
}

impl<'a> ReportLeaptofrogans<'a> {
    /// Build one over a borrowed address (the producer / client side).
    pub fn new(address: &'a str) -> Self {
        ReportLeaptofrogans { address }
    }

    /// Decode an inbound payload, borrowing its address for the call (the
    /// consumer / server side).
    pub fn from_raw(raw: &'a Raw) -> Self {
        // SAFETY: `raw.address` is valid for the duration of the call that
        // delivered `raw`, which is `'a`.
        ReportLeaptofrogans {
            address: unsafe { as_str(raw.address) },
        }
    }

    /// Encode into the raw payload, pointing a descriptor at our borrow (the
    /// producer / client side).
    pub fn to_raw(&self) -> Raw {
        Raw {
            event_id: EVT_LEAPTOFROGANS,
            address: ustring(self.address),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_a_borrow() {
        // Producer side: hold a borrow, encode to raw.
        let evt = ReportLeaptofrogans::new("frogans*example");
        let raw = evt.to_raw();

        // Consumer side: decode that raw back, borrowing the same bytes.
        let back = ReportLeaptofrogans::from_raw(&raw);
        assert_eq!(back.address, "frogans*example");
    }
}
