//! Per-`ctx` engine state: the command **outbox**.
//!
//! Each staged command carries the [`CallPool`] its data lives in — the 1:1
//! pool-per-command model: a command is consumed exactly once (at its `_pop`), so
//! it has exactly one pool, which becomes its `mempool_out` handle. The outbox is
//! shared (`Arc<Mutex<…>>`) so a background [`Sender`](crate::Sender) can stage
//! commands from another thread.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use fprt_core::Command;

use crate::encoder::CallPool;

/// A command waiting to be popped, paired with the pool its data lives in.
/// The pool stays empty (→ `EMPTY` handle) for pool-free commands.
pub struct Staged {
    /// The command the engine emitted.
    pub command: Command,
    /// The command's own pool — its data, and its eventual `mempool_out` handle.
    pub pool: CallPool,
}

/// A per-`ctx` FIFO queue of staged commands, drained by the pop sequence.
/// Cloning shares the same queue (that's how the cross-thread sender works).
#[derive(Clone, Default)]
pub struct Outbox(Arc<Mutex<VecDeque<Staged>>>);

impl Outbox {
    /// A new, empty outbox.
    pub fn new() -> Self {
        Outbox(Arc::new(Mutex::new(VecDeque::new())))
    }

    /// Stage a command at the back of the queue.
    pub fn push(&self, staged: Staged) {
        self.lock().push_back(staged);
    }

    /// Take the front staged command, or `None` if the queue is empty.
    pub fn pop_front(&self) -> Option<Staged> {
        self.lock().pop_front()
    }

    /// Read something off the front command without removing it — what
    /// `get_next_command` uses to report the next command's id while leaving it in
    /// place for the matching `_pop` to take.
    pub fn with_front<R>(&self, f: impl FnOnce(&Command) -> R) -> Option<R> {
        self.lock().front().map(|staged| f(&staged.command))
    }

    /// Whether the queue is currently empty.
    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// Number of staged commands.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, VecDeque<Staged>> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn staged(command: Command) -> Staged {
        Staged {
            command,
            pool: CallPool::new(),
        }
    }

    #[test]
    fn drains_fifo() {
        let out = Outbox::new();
        out.push(staged(Command::MenuOpen));
        out.push(staged(Command::MenuShow));
        out.push(staged(Command::MenuClose));
        assert_eq!(out.len(), 3);
        assert!(matches!(out.pop_front().unwrap().command, Command::MenuOpen));
        assert!(matches!(out.pop_front().unwrap().command, Command::MenuShow));
        assert!(matches!(out.pop_front().unwrap().command, Command::MenuClose));
        assert!(out.pop_front().is_none());
    }

    #[test]
    fn cross_thread_staging() {
        let out = Outbox::new();
        let worker = out.clone();
        thread::spawn(move || {
            worker.push(staged(Command::PadOpen));
            worker.push(staged(Command::PadShow));
        })
        .join()
        .unwrap();
        assert_eq!(out.len(), 2);
        assert!(matches!(out.pop_front().unwrap().command, Command::PadOpen));
        assert!(matches!(out.pop_front().unwrap().command, Command::PadShow));
    }

    #[test]
    fn outboxes_are_independent() {
        let a = Outbox::new();
        let b = Outbox::new();
        a.push(staged(Command::MenuOpen));
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 0);
    }
}
