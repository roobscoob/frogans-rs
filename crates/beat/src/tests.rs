//! Behavioural tests for the four core promises: snapshot scheduling, timers, channel
//! recv parking, and channels-over-channels (the http request/reply shape).

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use crate::{Beat, Either, Sender, Tick, select, yield_now};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A task spawned during a beat does not run until the next one; a still-pending task
/// is re-polled every beat.
#[test]
fn spawn_waits_for_next_beat() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    let (l, hc) = (log.clone(), h.clone());
    h.spawn(async move {
        l.borrow_mut().push("parent");
        let l2 = l.clone();
        hc.spawn(async move { l2.borrow_mut().push("child") });
    });

    rt.beat(Duration::ZERO);
    assert_eq!(*log.borrow(), ["parent"], "child is queued for next beat");
    rt.beat(Duration::ZERO);
    assert_eq!(*log.borrow(), ["parent", "child"]);
}

/// A timer reports `In(remaining)` each beat and fires once the clock reaches it.
#[test]
fn timer_counts_down_then_fires() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let done = Rc::new(Cell::new(false));

    let (hc, d) = (h.clone(), done.clone());
    h.spawn(async move {
        hc.after(ms(100)).await;
        d.set(true);
    });

    assert_eq!(rt.beat(ms(0)), Tick::In(ms(100)), "deadline set on first poll");
    assert_eq!(rt.beat(ms(50)), Tick::In(ms(50)));
    assert_eq!(rt.beat(ms(100)), Tick::Idle, "fires, task completes");
    assert!(done.get());
}

/// An empty recv parks the task and reports `Soon`; a send unblocks it next beat.
#[test]
fn recv_parks_soon_then_resolves() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (tx, rx) = h.channel::<u32>();
    let got = Rc::new(Cell::new(0u32));

    let g = got.clone();
    h.spawn(async move {
        g.set(rx.recv().await);
    });

    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon, "parked on recv");
    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon, "still parked, re-polled");
    tx.send(7);
    assert_eq!(rt.beat(Duration::ZERO), Tick::Idle, "message delivered");
    assert_eq!(got.get(), 7);
}

/// The motivating shape: build a reply channel, mail its `Sender` to the "io side"
/// inside a request, await the reply. Here the io side is driven by hand.
#[test]
fn channels_over_channels_request_reply() {
    let mut rt = Beat::new();
    let h = rt.handle();

    // The io request channel carries `(arg, reply-sender)`.
    let (req_tx, req_rx) = h.channel::<(u32, Sender<u32>)>();
    let answer = Rc::new(Cell::new(None));

    let (hc, a) = (h.clone(), answer.clone());
    h.spawn(async move {
        let (rep_tx, rep_rx) = hc.channel::<u32>();
        req_tx.send((21, rep_tx)); // hand the reply channel to the io side
        a.set(Some(rep_rx.recv().await));
    });

    // Beat 1: task sends the request and parks on the reply.
    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon);
    assert_eq!(answer.get(), None);

    // The io side: take the request, reply with the doubled value.
    let (arg, reply) = req_rx.try_recv().expect("request enqueued");
    reply.send(arg * 2);

    // Beat 2: the reply is waiting, the task completes.
    assert_eq!(rt.beat(Duration::ZERO), Tick::Idle);
    assert_eq!(answer.get(), Some(42));
}

/// A real second thread as the io side: the `Sender` crosses the thread boundary and
/// fires the `Notify` hook when it replies.
#[test]
fn sender_crosses_threads_and_notifies() {
    let woken = Arc::new(AtomicBool::new(false));
    let w = woken.clone();
    let mut rt = Beat::with_notify(crate::Notify::new(move || w.store(true, Ordering::SeqCst)));
    let h = rt.handle();

    let (rep_tx, rep_rx) = h.channel::<u32>();
    let answer = Rc::new(Cell::new(0));
    let a = answer.clone();
    h.spawn(async move { a.set(rep_rx.recv().await) });

    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon);

    // Hand the sender to another thread; it replies and fires Notify.
    std::thread::spawn(move || rep_tx.send(99)).join().unwrap();
    assert!(woken.load(Ordering::SeqCst), "cross-thread send fired the hook");

    assert_eq!(rt.beat(Duration::ZERO), Tick::Idle);
    assert_eq!(answer.get(), 99);
}

/// `yield_now` runs one loop step per beat, reporting `Soon` until the loop ends.
#[test]
fn yield_now_resumes_next_beat() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let steps = Rc::new(Cell::new(0u32));

    let s = steps.clone();
    h.spawn(async move {
        for _ in 0..3 {
            s.set(s.get() + 1);
            yield_now().await;
        }
    });

    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon);
    assert_eq!(steps.get(), 1);
    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon);
    assert_eq!(steps.get(), 2);
    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon);
    assert_eq!(steps.get(), 3);
    assert_eq!(rt.beat(Duration::ZERO), Tick::Idle, "loop ends, nothing pending");
    assert_eq!(steps.get(), 3);
}

/// `select` returns whichever branch has a message; the loser is dropped.
#[test]
fn select_takes_the_ready_branch() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (tx_a, rx_a) = h.channel::<&'static str>();
    let (_tx_b, rx_b) = h.channel::<&'static str>();
    let winner = Rc::new(Cell::new(""));

    let w = winner.clone();
    h.spawn(async move {
        let pick = match select(rx_a.recv(), rx_b.recv()).await {
            Either::Left(s) => s,
            Either::Right(s) => s,
        };
        w.set(pick);
    });

    assert_eq!(rt.beat(Duration::ZERO), Tick::Soon, "both branches empty");
    tx_a.send("a");
    assert_eq!(rt.beat(Duration::ZERO), Tick::Idle);
    assert_eq!(winner.get(), "a");
}

/// A timeout that fires completes the task cleanly — and the recv branch polled on the
/// way leaves no spurious `Soon` (the per-task accounting fix).
#[test]
fn timeout_elapses_cleanly() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (_tx, rx) = h.channel::<u32>();
    let timed_out = Rc::new(Cell::new(None));

    let (hc, t) = (h.clone(), timed_out.clone());
    h.spawn(async move {
        t.set(Some(hc.timeout(ms(100), rx.recv()).await.is_err()));
    });

    assert_eq!(rt.beat(ms(0)), Tick::Soon);
    assert_eq!(rt.beat(ms(50)), Tick::Soon);
    assert_eq!(rt.beat(ms(100)), Tick::Idle, "timer wins, no leftover Soon");
    assert_eq!(timed_out.get(), Some(true));
}

/// A result that arrives before the deadline is kept (left-biased select).
#[test]
fn timeout_keeps_in_time_result() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (tx, rx) = h.channel::<u32>();
    let got = Rc::new(Cell::new(None));

    let (hc, g) = (h.clone(), got.clone());
    h.spawn(async move {
        g.set(Some(hc.timeout(ms(100), rx.recv()).await));
    });

    assert_eq!(rt.beat(ms(0)), Tick::Soon);
    tx.send(5);
    assert_eq!(rt.beat(ms(10)), Tick::Idle);
    assert_eq!(got.get(), Some(Ok(5)));
}

/// `settle` runs a whole spawn chain to quiescence in one call — `beat` alone would
/// advance it only one link per call.
#[test]
fn settle_drains_a_spawn_chain_in_one_turn() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    let (l, hc) = (log.clone(), h.clone());
    h.spawn(async move {
        l.borrow_mut().push("a");
        let (l2, hc2) = (l.clone(), hc.clone());
        hc.spawn(async move {
            l2.borrow_mut().push("b");
            let l3 = l2.clone();
            hc2.spawn(async move { l3.borrow_mut().push("c") });
        });
    });

    assert_eq!(rt.settle(Duration::ZERO), Tick::Idle);
    assert_eq!(*log.borrow(), ["a", "b", "c"]);
}

/// An intra-engine `send`/`recv` chain resolves within a single `settle`, regardless of
/// task order.
#[test]
fn settle_resolves_intra_engine_messages() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (tx, rx) = h.channel::<u32>();
    let (tx2, rx2) = h.channel::<u32>();
    let out = Rc::new(Cell::new(0u32));

    // B: double what it receives and forward; C: store what B forwards.
    h.spawn(async move {
        let n = rx.recv().await;
        tx2.send(n * 2);
    });
    let o = out.clone();
    h.spawn(async move { o.set(rx2.recv().await) });

    tx.send(5); // prime the chain
    assert_eq!(rt.settle(Duration::ZERO), Tick::Idle);
    assert_eq!(out.get(), 10);
}

/// A `send` reaches a same-turn consumer even when the producer is polled *after* the
/// consumer has already parked and the producer never completes (a long-lived actor).
/// This is the case completion-tracking would miss: dirtying on the send, not on the
/// producer finishing, is what delivers it within the turn.
#[test]
fn settle_delivers_a_send_from_a_reparking_producer() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (tx, rx) = h.channel::<u32>();
    let (_keep, idle_rx) = h.channel::<()>(); // producer parks here, never resolves
    let got = Rc::new(Cell::new(None));

    // Consumer spawned first → polled before the producer each beat.
    let g = got.clone();
    h.spawn(async move { g.set(Some(rx.recv().await)) });
    // Producer sends, then parks forever without completing.
    h.spawn(async move {
        tx.send(7);
        let _ = idle_rx.recv().await;
    });

    assert_eq!(rt.settle(Duration::ZERO), Tick::Soon, "producer still parked");
    assert_eq!(got.get(), Some(7), "delivered within the same turn");
}

/// A task parked on an io channel makes `settle` return `Soon` after a single beat — it
/// does not spin waiting for the off-thread reply.
#[test]
fn settle_stops_at_an_io_block() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let (_tx, rx) = h.channel::<u32>();
    h.spawn(async move {
        let _ = rx.recv().await;
    });

    assert_eq!(rt.settle(Duration::ZERO), Tick::Soon);
}

/// `yield_now` does not extend a turn: each `settle` advances one loop step and reports
/// `Soon`, so the task resumes on the next turn rather than spinning this one.
#[test]
fn settle_treats_yield_as_next_turn() {
    let mut rt = Beat::new();
    let h = rt.handle();
    let steps = Rc::new(Cell::new(0u32));

    let s = steps.clone();
    h.spawn(async move {
        for _ in 0..3 {
            s.set(s.get() + 1);
            yield_now().await;
        }
    });

    assert_eq!(rt.settle(Duration::ZERO), Tick::Soon);
    assert_eq!(steps.get(), 1);
    assert_eq!(rt.settle(Duration::ZERO), Tick::Soon);
    assert_eq!(steps.get(), 2);
    assert_eq!(rt.settle(Duration::ZERO), Tick::Soon);
    assert_eq!(steps.get(), 3);
    assert_eq!(rt.settle(Duration::ZERO), Tick::Idle, "loop ended");
    assert_eq!(steps.get(), 3);
}

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
