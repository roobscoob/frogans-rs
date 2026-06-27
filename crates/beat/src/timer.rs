//! [`Timer`] — a future that completes once the injected clock reaches its deadline.
//!
//! There is no timer wheel and no waker: a pending timer just records how long it
//! still has into the current beat's context, and the executor folds those into the
//! [`Tick::In`](crate::Tick) it returns. The deadline is fixed on the *first* poll
//! (when the task actually reaches the `.await`), relative to that beat's `now`.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// A timeout. Build one with [`Handle::after`](crate::Handle::after) and `.await` it.
/// Idle until first polled; from then it is due `dur` after that beat's clock.
#[derive(Debug)]
pub struct Timer {
    dur: Duration,
    /// `now + dur`, captured on first poll. `None` until then.
    deadline: Option<Duration>,
}

impl Timer {
    pub(crate) fn new(dur: Duration) -> Self {
        Timer { dur, deadline: None }
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        // Timer holds no self-references, so plain `get_mut` is fine.
        let this = self.get_mut();
        // Outside a beat there is no clock; treat as not-yet-due rather than panic.
        let now = crate::ctx::now().unwrap_or(Duration::ZERO);
        let deadline = *this.deadline.get_or_insert(now + this.dur);
        if now >= deadline {
            Poll::Ready(())
        } else {
            crate::ctx::register_timer(deadline - now);
            Poll::Pending
        }
    }
}
