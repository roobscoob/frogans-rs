//! [`Selection`] — the borrowed address/label list carried by the dialog list
//! events (favorites / recentlyvisited / blocked / recovery / devtools
//! open/remove/delete/inspect).
//!
//! Shared because every one has the same `AddressSelection` shape: a count plus an
//! array of `Ustring`s. Decode-only here (`from_raw`); the client sends these via
//! its own `address_selection_event!`, so there's no `to_raw` to mirror.

use fprt_sys::ui::AddressSelection as Raw;

use crate::wire::as_str;

/// The entries the user selected in a dialog list event — a borrowed view of the
/// inbound address/label array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<'a> {
    /// The selected entries, in order (addresses or labels, per the event).
    pub addresses: Vec<&'a str>,
}

impl<'a> Selection<'a> {
    /// Build one over borrowed entries.
    pub fn new(addresses: Vec<&'a str>) -> Self {
        Selection { addresses }
    }

    /// Decode the inbound selection, borrowing each entry for the call.
    pub fn from_raw(raw: &'a Raw) -> Self {
        let mut addresses = Vec::with_capacity(raw.count as usize);
        if !raw.items.is_null() {
            for i in 0..raw.count as usize {
                // SAFETY: `items` points at `count` `Ustring`s, each valid for the
                // call's duration (host contract).
                let entry = unsafe { *raw.items.add(i) };
                addresses.push(unsafe { as_str(entry) });
            }
        }
        Selection { addresses }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::ustring;
    use fprt_sys::ui::EventTag;

    #[test]
    fn decodes_the_entries() {
        let items = [ustring("frogans*a"), ustring("frogans*b")];
        let raw = Raw {
            event_id: EventTag(0),
            _rsv04: 0,
            count: items.len() as u32,
            items: items.as_ptr(),
        };
        assert_eq!(Selection::from_raw(&raw).addresses, ["frogans*a", "frogans*b"]);
    }

    #[test]
    fn empty_is_empty() {
        let raw = Raw {
            event_id: EventTag(0),
            _rsv04: 0,
            count: 0,
            items: core::ptr::null(),
        };
        assert!(Selection::from_raw(&raw).addresses.is_empty());
    }
}
