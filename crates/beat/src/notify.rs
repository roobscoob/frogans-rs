//! [`Notify`] — the one cross-thread seam.
//!
//! `beat` is sans-io: it never blocks and never owns a wait primitive. But a channel
//! [`Sender`](crate::Sender) can ride to another thread (the io side), and when that
//! thread sends, *something* has to tell the engine's driver to beat again. That
//! "something" is a `Notify` hook: a send pushes the message, then calls the hook.
//!
//! `beat` defines the hook but supplies only [`Notify::noop`]; the driver (the io
//! crate) installs a real one — typically "unpark my loop" / "set my OS event" /
//! "ask the FPRT host for a turn". A pure-`beat` test uses the no-op and just calls
//! [`beat`](crate::Beat::beat) in a loop.

use std::fmt;
use std::sync::Arc;

/// A cross-thread wake hook fired by every channel send. Cloneable and `Send`/`Sync`
/// so it travels with a [`Sender`](crate::Sender). Calling [`wake`](Notify::wake) is
/// the driver's cue to run another beat; `beat` itself never waits on it.
#[derive(Clone)]
pub struct Notify(Arc<dyn Fn() + Send + Sync>);

impl Notify {
    /// A hook that runs `f` on every send (e.g. unpark the driver thread).
    pub fn new(f: impl Fn() + Send + Sync + 'static) -> Self {
        Notify(Arc::new(f))
    }

    /// The do-nothing hook — for engine-only graphs and tests that drive `beat()`
    /// by hand.
    pub fn noop() -> Self {
        Notify(Arc::new(|| {}))
    }

    /// Fire the hook: a send happened, the driver should beat again.
    pub fn wake(&self) {
        (self.0)()
    }
}

impl Default for Notify {
    fn default() -> Self {
        Notify::noop()
    }
}

impl fmt::Debug for Notify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Notify(..)")
    }
}
