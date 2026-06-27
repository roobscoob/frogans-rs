//! Bridging `beat` to the FPRT turn — mapping a beat's [`Tick`] to the [`NextWake`]
//! the engine reports at `sync_leave`.
//!
//! The engine is *reactive*: the host grants a conductor a turn only on its own
//! schedule — when it has a UI event, or when the `In(ms)` delay the engine last asked
//! for elapses. There is no out-of-band wake; the engine cannot tell a foreign host to
//! come back *now*. So when a beat ends, the conductor translates the executor's
//! intent into that single `In(ms)`-or-`Idle` lever.
//!
//! The interesting case is [`Tick::Soon`]: a task is parked on a channel an off-thread
//! (io) sender may fill at any moment. We *cannot* report [`NextWake::Idle`] — if no UI
//! event happens to arrive, the host would never return and the io reply would
//! deadlock. So `Soon` maps to a finite poll cadence ([`Pacing::soon`]); the channel
//! [`Notify`](beat::Notify) remains the seam by which an embedder we control wakes
//! earlier than that cadence.
//!
//! ```
//! use std::time::Duration;
//! use beat::Tick;
//! use fprt_impl::runtime::Pacing;
//! use fprt_server::NextWake;
//!
//! let pacing = Pacing::default();
//! assert_eq!(pacing.next_wake(Tick::Idle), NextWake::Idle);
//! assert_eq!(pacing.next_wake(Tick::In(Duration::from_millis(100))), NextWake::In(Duration::from_millis(100)));
//! // Soon becomes a finite recheck — never Idle (that would deadlock on io).
//! assert_ne!(pacing.next_wake(Tick::Soon), NextWake::Idle);
//! ```

use std::time::Duration;

use beat::Tick;
use fprt_server::NextWake;

/// The default recheck cadence for [`Tick::Soon`] — 4ms (≈250 turns/s). Responsive for
/// network/ipc io, whose latency dwarfs it, while bounding idle CPU. Only an upper
/// bound: an embedder may wake sooner off the channel [`Notify`](beat::Notify).
pub const DEFAULT_SOON: Duration = Duration::from_millis(4);

/// How a conductor turns a beat outcome into a [`NextWake`]. Holds the [`Tick::Soon`]
/// poll cadence; everything else is a direct translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pacing {
    /// The delay reported for [`Tick::Soon`] — the cadence at which the host rechecks
    /// channels an off-thread sender may have filled.
    pub soon: Duration,
}

impl Pacing {
    /// A pacing with an explicit `Soon` cadence.
    pub fn new(soon: Duration) -> Self {
        Pacing { soon }
    }

    /// Map a beat outcome to the engine's next-wake.
    ///
    /// - [`Tick::Idle`] → [`NextWake::Idle`] (wake on the next external event).
    /// - [`Tick::In(d)`](Tick::In) → [`NextWake::In(d)`](NextWake::In) (a timer is due).
    /// - [`Tick::Soon`] → [`NextWake::In`]`(self.soon)` — finite, never `Idle`.
    pub fn next_wake(self, tick: Tick) -> NextWake {
        match tick {
            Tick::Idle => NextWake::Idle,
            Tick::In(d) => NextWake::In(d),
            Tick::Soon => NextWake::In(self.soon),
        }
    }
}

impl Default for Pacing {
    fn default() -> Self {
        Pacing { soon: DEFAULT_SOON }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_passes_through() {
        assert_eq!(Pacing::default().next_wake(Tick::Idle), NextWake::Idle);
    }

    #[test]
    fn timer_passes_through() {
        let d = Duration::from_millis(250);
        assert_eq!(Pacing::default().next_wake(Tick::In(d)), NextWake::In(d));
    }

    #[test]
    fn soon_becomes_the_poll_cadence() {
        let p = Pacing::new(Duration::from_millis(10));
        assert_eq!(p.next_wake(Tick::Soon), NextWake::In(Duration::from_millis(10)));
    }

    /// The load-bearing invariant: `Soon` is always a finite wake, never the idle
    /// sentinel — otherwise an io reply with no accompanying UI event deadlocks.
    #[test]
    fn soon_is_never_idle() {
        for soon in [Duration::ZERO, Duration::from_millis(4), Duration::from_secs(1)] {
            let wake = Pacing::new(soon).next_wake(Tick::Soon);
            assert_ne!(wake, NextWake::Idle);
            assert_ne!(wake.to_raw(), u32::MAX, "the wire word is a real ms, not idle");
        }
    }

    #[test]
    fn idle_is_the_sentinel_on_the_wire() {
        assert_eq!(Pacing::default().next_wake(Tick::Idle).to_raw(), u32::MAX);
    }
}
