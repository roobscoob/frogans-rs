//! Combinators over `beat` futures: [`select`] (race two), [`yield_now`] (cooperative
//! yield), and [`timeout`] (race against a [`Timer`]).
//!
//! All of these poll their branches once per beat under the no-waker model — there is
//! nothing to wake, so a combinator is just "poll the parts, see what's ready." Each
//! branch's poll feeds the per-task accounting (a pending recv flags
//! [`Soon`](crate::Tick), a pending timer registers its remaining time), and because
//! that accounting is committed only when the *task* stays pending, a combinator that
//! completes via one branch leaves no trace from the others.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::Timer;

/// Which branch of a two-way [`select`] finished first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Either<A, B> {
    /// The first future completed.
    Left(A),
    /// The second future completed.
    Right(B),
}

/// Race two futures; the first to finish wins and the other is dropped. **Left-biased**:
/// if both are ready on the same poll, [`Left`](Either::Left) wins.
///
/// Both futures must be [`Unpin`] — true of [`Recv`](crate::Recv), [`Timer`], and
/// [`YieldNow`]. To race an `async` block, `Box::pin` it first (a boxed future is
/// `Unpin`).
pub fn select<A, B>(a: A, b: B) -> Select<A, B>
where
    A: Future + Unpin,
    B: Future + Unpin,
{
    Select { a, b }
}

/// The future returned by [`select`].
#[derive(Debug)]
pub struct Select<A, B> {
    a: A,
    b: B,
}

impl<A, B> Future for Select<A, B>
where
    A: Future + Unpin,
    B: Future + Unpin,
{
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Both branches are Unpin, so the Select is too and `get_mut` is sound.
        let this = self.get_mut();
        if let Poll::Ready(v) = Pin::new(&mut this.a).poll(cx) {
            return Poll::Ready(Either::Left(v));
        }
        if let Poll::Ready(v) = Pin::new(&mut this.b).poll(cx) {
            return Poll::Ready(Either::Right(v));
        }
        Poll::Pending
    }
}

/// Yield once: pending on this beat (flagging it [`Soon`](crate::Tick)), ready on the
/// next. Lets a task give up the rest of the beat and resume next time.
pub fn yield_now() -> YieldNow {
    YieldNow { done: false }
}

/// The future returned by [`yield_now`].
#[derive(Debug)]
pub struct YieldNow {
    done: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        if this.done {
            Poll::Ready(())
        } else {
            this.done = true;
            crate::ctx::mark_soon();
            Poll::Pending
        }
    }
}

/// The error from [`timeout`]: the [`Timer`] fired before the future completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elapsed;

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("future timed out")
    }
}

impl std::error::Error for Elapsed {}

/// Run `fut` until it completes or `timer` fires, whichever comes first. `Ok` is the
/// future's output; `Err(Elapsed)` means the timer won. The future is favored on a tie
/// (`select` is left-biased), so a result that lands exactly on the deadline is kept.
///
/// Build the timer with [`Handle::after`](crate::Handle::after), or use the
/// [`Handle::timeout`](crate::Handle::timeout) shorthand.
pub async fn timeout<F>(timer: Timer, fut: F) -> Result<F::Output, Elapsed>
where
    F: Future + Unpin,
{
    match select(fut, timer).await {
        Either::Left(v) => Ok(v),
        Either::Right(()) => Err(Elapsed),
    }
}
