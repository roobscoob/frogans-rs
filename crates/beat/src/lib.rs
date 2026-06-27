//! `beat` — a sans-io, single-threaded cooperative executor.
//!
//! One call, [`Beat::beat`], drives the whole world. It takes a snapshot of the tasks
//! that exist *now*, polls each exactly once (a poll runs a task to its next pending
//! await), and returns a [`Tick`] saying when the driver should beat again. Tasks
//! spawned *during* a beat first run on the next one; tasks that are still pending are
//! re-polled every beat. There are **no wakers** — progress is made only by beating.
//!
//! ```
//! use std::time::Duration;
//! use beat::{Beat, Tick};
//!
//! let mut rt = Beat::new();
//! let h = rt.handle();
//! let (tx, rx) = h.channel::<u32>();
//! h.spawn(async move {
//!     let n = rx.recv().await;        // parks until a message arrives
//!     assert_eq!(n, 42);
//! });
//!
//! assert_eq!(rt.beat(Duration::ZERO), Tick::Soon);   // task is parked on recv
//! tx.send(42);
//! assert_eq!(rt.beat(Duration::ZERO), Tick::Idle);   // task completed
//! ```
//!
//! ## How a leaf future reports *why* it is pending
//!
//! [`Future::poll`] only hands you a [`Waker`], which a sans-waker executor cannot
//! use. So for the duration of a beat we install a thread-local [`ctx`] and poll with
//! [`Waker::noop`]. A pending [`Timer`] records how long it still has; a pending
//! [`Recv`](channel::Recv) flips a "soon" flag. After polling, the [`Tick`] falls out
//! of those: any soon → [`Soon`](Tick::Soon), else the nearest timer →
//! [`In`](Tick::In), else [`Idle`](Tick::Idle).
//!
//! ## Threads
//!
//! Everything here is `Rc`-based and single-threaded. The *only* part that crosses to
//! the io side is a channel endpoint (the queue is `Arc`/`Mutex`); a cross-thread
//! [`send`](Sender::send) fires a [`Notify`] to prompt the driver to beat again.

mod channel;
mod combinator;
mod notify;
mod timer;

pub use channel::{Receiver, Recv, Sender};
pub use combinator::{Either, Elapsed, Select, YieldNow, select, timeout, yield_now};
pub use notify::Notify;
pub use timer::Timer;

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Waker};
use std::time::Duration;

/// A spawned task: a pinned, heap-allocated future to completion. Not `Send` —
/// engine tasks may hold `Rc` and other thread-affine state.
type Task = Pin<Box<dyn Future<Output = ()>>>;

/// What a beat tells its driver about the next one.
///
/// Maps onto how the driver should wait: block until a cross-thread [`Notify`] for
/// [`Soon`], sleep at most the duration for [`In`](Tick::In), or sleep until an
/// external event for [`Idle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    /// A task is ready to run again as soon as possible — parked on a channel recv
    /// (a message may arrive from the io side) or having yielded. Beat again when
    /// notified.
    Soon,
    /// No soon-waiters, but the nearest timer is due in at most this long.
    In(Duration),
    /// Nothing is pending; only a new spawn or a cross-thread send makes more work.
    Idle,
}

/// The injected clock plus the [`Tick`] accounting, shared with leaf futures through a
/// thread-local for the duration of a single [`Beat::beat`] call.
///
/// Accounting is two-level. Leaf futures write to a *per-task* scratch
/// ([`mark_soon`]/[`register_timer`]); the beat loop resets it before each task poll
/// and folds it into the beat-wide result **only if that task stayed pending**. So a
/// task that completes — even one that polled a recv on the way (e.g. a `select` whose
/// timeout branch won) — contributes nothing to the returned [`Tick`].
pub(crate) mod ctx {
    use std::cell::RefCell;
    use std::time::Duration;

    /// What the executor exposes to leaf futures during a beat.
    pub(crate) struct BeatCtx {
        /// The clock the driver handed to this beat.
        pub now: Duration,
        /// Beat-wide: any *pending* task parked for immediate re-poll (recv / yield).
        pub soon: bool,
        /// Beat-wide: smallest remaining time across *pending* tasks' timers.
        pub min_timer: Option<Duration>,
        /// Beat-wide: this beat created work another task can act on — a channel
        /// `send` or a `spawn`. Those are the only two ways one task unblocks another,
        /// so they alone decide whether [`settle`](crate::Beat::settle) beats again. A
        /// bare `yield_now`, a consumed message, a completion, or a fired timer do
        /// *not* set it (a yield resumes next turn; the rest enable no other task
        /// except through the sends/spawns they already trigger).
        pub dirty: bool,
        /// Scratch for the task being polled now — merged into the beat-wide fields
        /// only if the task returns pending.
        task_soon: bool,
        task_timer: Option<Duration>,
    }

    impl BeatCtx {
        pub fn new(now: Duration) -> Self {
            BeatCtx {
                now,
                soon: false,
                min_timer: None,
                dirty: false,
                task_soon: false,
                task_timer: None,
            }
        }
    }

    fn min_merge(slot: &mut Option<Duration>, d: Duration) {
        *slot = Some(match *slot {
            Some(m) => m.min(d),
            None => d,
        });
    }

    thread_local! {
        static CURRENT: RefCell<Option<BeatCtx>> = const { RefCell::new(None) };
    }

    /// Install `ctx` for the current beat, returning the previous one (always `None`
    /// in practice — beats do not nest — but restored for safety).
    pub(crate) fn install(ctx: BeatCtx) -> Option<BeatCtx> {
        CURRENT.with(|c| c.borrow_mut().replace(ctx))
    }

    /// Remove and return the current beat's ctx.
    pub(crate) fn take() -> Option<BeatCtx> {
        CURRENT.with(|c| c.borrow_mut().take())
    }

    fn with<R>(f: impl FnOnce(&mut BeatCtx) -> R) -> Option<R> {
        CURRENT.with(|c| c.borrow_mut().as_mut().map(f))
    }

    /// Clear the per-task scratch before polling a task.
    pub(crate) fn reset_task() {
        with(|c| {
            c.task_soon = false;
            c.task_timer = None;
        });
    }

    /// Fold the just-polled task's scratch into the beat-wide result, but only if it
    /// stayed `pending` (a completed task wants nothing).
    pub(crate) fn commit_task(pending: bool) {
        with(|c| {
            if pending {
                c.soon |= c.task_soon;
                if let Some(d) = c.task_timer {
                    min_merge(&mut c.min_timer, d);
                }
            }
        });
    }

    /// The clock for the in-progress beat, or `None` if polled outside one.
    pub(crate) fn now() -> Option<Duration> {
        with(|c| c.now)
    }

    /// Flag the current task as wanting an immediate re-poll → [`Soon`](crate::Tick).
    pub(crate) fn mark_soon() {
        with(|c| c.task_soon = true);
    }

    /// Fold a pending timer's remaining time into the current task's scratch minimum.
    pub(crate) fn register_timer(remaining: Duration) {
        with(|c| min_merge(&mut c.task_timer, remaining));
    }

    /// Record that this beat created work for another task — a channel `send` or a
    /// `spawn` — so [`settle`](crate::Beat::settle) beats again. A no-op outside a beat
    /// (an off-thread io send has no ctx installed), which keeps `settle` deterministic.
    pub(crate) fn mark_dirty() {
        with(|c| c.dirty = true);
    }
}

/// State shared between a [`Beat`] and every [`Handle`] cloned from it: the run queue,
/// the injected clock, and the cross-thread wake hook.
struct Shared {
    /// Tasks to run on the next beat — pending tasks re-queued plus anything spawned.
    incoming: RefCell<Vec<Task>>,
    /// The clock from the most recent beat (for inspection; timers read the live ctx).
    now: Cell<Duration>,
    /// Fired by channel sends; handed to every [`Sender`].
    notify: Notify,
}

/// The executor. Build one with [`Beat::new`], wire up tasks/channels/timers through a
/// [`handle`](Beat::handle), then drive it with [`beat`](Beat::beat).
pub struct Beat {
    shared: Rc<Shared>,
}

impl Beat {
    /// A fresh executor with a no-op [`Notify`] — the engine-only / test default.
    pub fn new() -> Self {
        Beat::with_notify(Notify::noop())
    }

    /// A fresh executor whose channel sends fire `notify` (the driver's wake hook).
    pub fn with_notify(notify: Notify) -> Self {
        Beat {
            shared: Rc::new(Shared {
                incoming: RefCell::new(Vec::new()),
                now: Cell::new(Duration::ZERO),
                notify,
            }),
        }
    }

    /// A cloneable handle for spawning, timers, and channels. Hand it into tasks.
    pub fn handle(&self) -> Handle {
        Handle { shared: self.shared.clone() }
    }

    /// The wake hook these channels fire — clone it into the driver if it needs to
    /// observe cross-thread sends.
    pub fn notify(&self) -> Notify {
        self.shared.notify.clone()
    }

    /// Spawn a task. It first runs on the *next* beat (see [`Beat::beat`]).
    pub fn spawn(&self, fut: impl Future<Output = ()> + 'static) {
        self.handle().spawn(fut);
    }

    /// Drive one beat: poll every task that exists now, exactly once, with `now` as the
    /// clock. Tasks spawned during the beat are queued for the next; still-pending
    /// tasks are re-queued. Returns when the driver should beat again.
    ///
    /// For a whole turn, prefer [`settle`](Beat::settle), which beats until the engine
    /// quiesces so spawned handlers and intra-engine message chains resolve at once.
    pub fn beat(&mut self, now: Duration) -> Tick {
        self.beat_once(now).0
    }

    /// Beat until the engine quiesces, then return the final [`Tick`] — the per-turn
    /// driver.
    ///
    /// Each beat that makes *progress* — a message consumed, a timer fired, a task
    /// completed or spawned — is followed by another, so a handler spawned at the start
    /// of the turn runs to its first blocking await, and intra-engine `send`/`recv`
    /// chains resolve, all within this one call. A beat that makes no progress is a
    /// fixpoint (the clock is fixed for the turn), so `settle` stops and reports what is
    /// still pending: [`Soon`](Tick::Soon) if parked on io, [`In`](Tick::In) for a
    /// timer, else [`Idle`](Tick::Idle).
    ///
    /// A bare [`yield_now`](crate::yield_now) is intentionally *not* progress, so it
    /// resumes next turn rather than spinning this one — `settle` always terminates on
    /// cooperative tasks. The cap is only a backstop against a task that spawns or
    /// consumes without bound.
    pub fn settle(&mut self, now: Duration) -> Tick {
        const CAP: usize = 1024;
        for _ in 0..CAP {
            let (tick, progress) = self.beat_once(now);
            if !progress {
                return tick;
            }
        }
        debug_assert!(false, "beat::settle exceeded {CAP} beats without quiescing");
        self.beat_once(now).0
    }

    /// One beat: poll each existing task once. Returns the resulting [`Tick`] and
    /// whether the beat made progress (drives [`settle`](Beat::settle)).
    fn beat_once(&mut self, now: Duration) -> (Tick, bool) {
        self.shared.now.set(now);

        // Snapshot the existing tasks. Pulling them out of `incoming` leaves it free
        // for spawns that happen during polling — those become next beat's work.
        let run = std::mem::take(&mut *self.shared.incoming.borrow_mut());

        let prev = ctx::install(ctx::BeatCtx::new(now));
        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        for mut task in run {
            // A poll runs the task to its next pending await. Ready → drop it; Pending
            // → keep it for the next beat. The per-task scratch is reset before and
            // committed after, so only a still-pending task shapes the `Tick`.
            ctx::reset_task();
            let pending = task.as_mut().poll(&mut cx).is_pending();
            ctx::commit_task(pending);
            if pending {
                self.shared.incoming.borrow_mut().push(task);
            }
        }

        let ctx = ctx::take().expect("beat ctx installed for the duration of the beat");
        if let Some(prev) = prev {
            ctx::install(prev);
        }

        // Progress = this beat created work another task can act on (a send or spawn);
        // that alone warrants another beat within the turn.
        let progress = ctx.dirty;

        let tick = if ctx.soon {
            Tick::Soon
        } else if let Some(d) = ctx.min_timer {
            Tick::In(d)
        } else {
            Tick::Idle
        };
        (tick, progress)
    }
}

impl Default for Beat {
    fn default() -> Self {
        Beat::new()
    }
}

/// A cloneable handle into a [`Beat`]: spawn tasks, make timers, open channels. Lives
/// on the engine thread (it is `!Send`).
#[derive(Clone)]
pub struct Handle {
    shared: Rc<Shared>,
}

impl Handle {
    /// Spawn a task. It first runs on the next beat. A spawn from inside a beat dirties
    /// it (new work to run), so [`settle`](Beat::settle) beats again.
    pub fn spawn(&self, fut: impl Future<Output = ()> + 'static) {
        self.shared.incoming.borrow_mut().push(Box::pin(fut));
        ctx::mark_dirty();
    }

    /// A timeout future, due `dur` after it is first polled. `.await` it.
    pub fn after(&self, dur: Duration) -> Timer {
        Timer::new(dur)
    }

    /// Run `fut` with a `dur` deadline: `Ok` on completion, `Err(Elapsed)` if `dur`
    /// passes first. Shorthand for [`timeout`]`(self.after(dur), fut)`.
    pub fn timeout<F>(&self, dur: Duration, fut: F) -> impl Future<Output = Result<F::Output, Elapsed>>
    where
        F: Future + Unpin,
    {
        combinator::timeout(self.after(dur), fut)
    }

    /// Open a channel whose sends fire this executor's [`Notify`].
    pub fn channel<T>(&self) -> (Sender<T>, Receiver<T>) {
        channel::channel(self.shared.notify.clone())
    }

    /// The clock from the most recent beat.
    pub fn now(&self) -> Duration {
        self.shared.now.get()
    }
}

#[cfg(test)]
mod tests;
