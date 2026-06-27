//! Host→engine events: the [`Report`] trait, implemented by the per-event types
//! in [`component`](super::component) and accepted by
//! [`Conductor::report`](super::Conductor::report).

use crate::error::EngineError;

use super::Conductor;

mod event;

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// A host→engine event.
///
/// **Sealed** — implemented only by this crate's per-event types (e.g.
/// [`ReportStart`](super::component::application::ReportStart)). Construct one and
/// pass it to [`Conductor::report`](super::Conductor::report); it fires inside the
/// turn's sync window.
///
/// Unlike [`Command`](super::Command) — a flat enum you *match* — events are
/// distinct types you *construct*, so there's no central enum to maintain: each
/// event lives entirely in its component module.
#[allow(private_bounds)]
pub trait Report: sealed::Sealed {
    /// Fire this event on `conductor` (host → engine). Called by
    /// [`Conductor::report`](super::Conductor::report) once the window is open.
    #[doc(hidden)]
    fn send(self, conductor: &Conductor) -> Result<(), EngineError>;
}

/// Define a no-data event (a `Report<EventTag>` signal):
/// `marker_event!(ReportCancel, fprt_sys::ui::menu::EVT_CANCEL, menu_cancel);`
/// generates the unit type, its `new()`, and its [`Report`] impl.
macro_rules! marker_event {
    ($name:ident, $tag:expr, $export:ident) => {
        /// A no-data event (host → engine).
        #[derive(Default)]
        pub struct $name;

        impl $name {
            /// Construct the event.
            pub fn new() -> Self {
                $name
            }
        }

        impl $crate::conductor::report::sealed::Sealed for $name {}

        impl $crate::conductor::report::Report for $name {
            fn send(self, conductor: &$crate::Conductor) -> Result<(), $crate::EngineError> {
                let engine = conductor.engine();
                let ctx = conductor.ctx();
                let tag = $tag;
                // SAFETY: valid ctx; `tag` outlives the call.
                $crate::call::invoke(engine, |s, e, p| unsafe {
                    (engine.methods().$export)(ctx, &tag, s, e, p)
                })
                .map(|_| ())
            }
        }
    };
}
pub(crate) use marker_event;

/// Define a host→engine event carrying a selected address/label list
/// (`Report<AddressSelection>`):
/// `address_selection_event!(ReportOpen, fprt_sys::ui::favorites::EVT_OPEN, favorites_open_event);`
/// The constructor borrows `&[&str]` for the call only.
macro_rules! address_selection_event {
    ($name:ident, $tag:expr, $export:ident) => {
        /// A host→engine event reporting the selected entries.
        pub struct $name<'a> {
            entries: &'a [&'a str],
        }

        impl<'a> $name<'a> {
            /// The selected entries (borrowed for the call only).
            pub fn new(entries: &'a [&'a str]) -> Self {
                $name { entries }
            }
        }

        impl $crate::conductor::report::sealed::Sealed for $name<'_> {}

        impl $crate::conductor::report::Report for $name<'_> {
            fn send(self, conductor: &$crate::Conductor) -> Result<(), $crate::EngineError> {
                let engine = conductor.engine();
                let ctx = conductor.ctx();
                let items: ::std::vec::Vec<::fprt_sys::ustring::Ustring> =
                    self.entries.iter().map(|s| $crate::call::ustring(s)).collect();
                let payload = ::fprt_sys::ui::AddressSelection {
                    event_id: $tag,
                    _rsv04: 0,
                    count: items.len() as u32,
                    items: items.as_ptr(),
                };
                // SAFETY: valid ctx; `payload`/`items`/`self.entries` outlive the call.
                $crate::call::invoke(engine, |s, e, p| unsafe {
                    (engine.methods().$export)(ctx, &payload, s, e, p)
                })
                .map(|_| ())
            }
        }
    };
}
pub(crate) use address_selection_event;
