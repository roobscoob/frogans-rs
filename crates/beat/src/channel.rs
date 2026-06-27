//! Channels — the engine's only cross-thread primitive.
//!
//! A channel is an `Arc`-shared MPSC queue: any number of [`Sender`]s (cloneable,
//! `Send`) feed one [`Receiver`]. The queue itself is thread-safe, so a `Sender` can
//! ride inside a message to the io thread — that is "channels over channels": the
//! payload type is just `(Request, Sender<Response>)`, and `Sender<Response>` is
//! `Send` whenever `Response` is.
//!
//! Receiving is beat-driven: [`Receiver::recv`] returns a future that, when the queue
//! is empty, flags the current beat as [`Soon`](crate::Tick::Soon) and yields. There
//! is no waker — the executor re-polls it next beat. A [`send`](Sender::send) fires
//! the channel's [`Notify`] so a cross-thread producer can prompt the driver to beat.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use crate::notify::Notify;

/// The shared queue behind a [`Sender`]/[`Receiver`] pair.
struct Chan<T> {
    queue: Mutex<VecDeque<T>>,
    notify: Notify,
}

/// The sending half. Cloneable; `Send` when `T` is — hand it to another thread or
/// mail it through another channel.
pub struct Sender<T> {
    chan: Arc<Chan<T>>,
}

/// The receiving half. Await messages with [`recv`](Receiver::recv) on the engine
/// thread, or poll non-blockingly with [`try_recv`](Receiver::try_recv).
pub struct Receiver<T> {
    chan: Arc<Chan<T>>,
}

/// Build a channel whose sends fire `notify`. Made via
/// [`Handle::channel`](crate::Handle::channel), which supplies the executor's hook.
pub(crate) fn channel<T>(notify: Notify) -> (Sender<T>, Receiver<T>) {
    let chan = Arc::new(Chan {
        queue: Mutex::new(VecDeque::new()),
        notify,
    });
    (
        Sender { chan: chan.clone() },
        Receiver { chan },
    )
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender { chan: self.chan.clone() }
    }
}

impl<T> Sender<T> {
    /// Enqueue a message and fire the channel's [`Notify`]. Never blocks.
    pub fn send(&self, msg: T) {
        self.chan.queue.lock().unwrap().push_back(msg);
        // A send made a message available to a parked receiver. From inside a beat that
        // dirties the turn so `settle` beats again; from the io thread (no beat) it is a
        // no-op here — the `Notify` is what wakes the driver instead.
        crate::ctx::mark_dirty();
        self.chan.notify.wake();
    }
}

impl<T> Receiver<T> {
    /// Take the next message if one is already queued, without awaiting.
    pub fn try_recv(&self) -> Option<T> {
        self.chan.queue.lock().unwrap().pop_front()
    }

    /// A future for the next message. Pending → flags the beat [`Soon`](crate::Tick).
    pub fn recv(&self) -> Recv<T> {
        Recv { chan: self.chan.clone() }
    }
}

/// The future returned by [`Receiver::recv`]. Holds its own `Arc`, so it carries no
/// borrow and is `'static`.
pub struct Recv<T> {
    chan: Arc<Chan<T>>,
}

impl<T> Future for Recv<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        match self.chan.queue.lock().unwrap().pop_front() {
            Some(msg) => Poll::Ready(msg),
            None => {
                crate::ctx::mark_soon();
                Poll::Pending
            }
        }
    }
}
