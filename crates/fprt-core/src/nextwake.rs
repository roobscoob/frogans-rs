//! [`NextWake`] — when the engine next wants a turn. Produced by the server
//! (the engine-fn's return), consumed by the client (`sync_leave`).

use core::time::Duration;

/// When the engine next wants a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextWake {
    /// Nothing pending — wake on the next external event.
    Idle,
    /// Wake again after this delay.
    In(Duration),
}

impl NextWake {
    /// Decode `sync_leave`'s next-wake word (`u32::MAX` ⇒ idle).
    pub fn from_raw(ms: u32) -> Self {
        if ms == u32::MAX {
            NextWake::Idle
        } else {
            NextWake::In(Duration::from_millis(ms as u64))
        }
    }

    /// Encode to the next-wake word the engine writes (`Idle` ⇒ `u32::MAX`; an
    /// over-long delay is clamped just below the idle sentinel).
    pub fn to_raw(self) -> u32 {
        match self {
            NextWake::Idle => u32::MAX,
            NextWake::In(d) => d.as_millis().min((u32::MAX - 1) as u128) as u32,
        }
    }
}
