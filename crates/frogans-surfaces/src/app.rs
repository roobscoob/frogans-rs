//! [`FrogansApp`] — the [`ApplicationHandler`] the embedder runs. It owns the
//! [`Session`] and the embedder, translates the winit lifecycle into engine turns,
//! and maps the engine's [`NextWake`] onto winit's [`ControlFlow`].

use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use crate::session::{Session, SessionConfig};
use crate::{Embedder, Frogans, NextWake};

/// The winit application driving an `fprt` session.
///
/// Construct one with a [`SessionConfig`] and an [`Embedder`], then hand it to
/// winit:
///
/// ```ignore
/// let event_loop = winit::event_loop::EventLoop::new()?;
/// let mut app = FrogansApp::new(config, my_embedder);
/// event_loop.run_app(&mut app)?;
/// ```
pub struct FrogansApp<E: Embedder> {
    config: SessionConfig,
    embedder: E,
    /// `None` until the first `resumed`, when the engine is brought up.
    session: Option<Session>,
    /// Guards against re-booting on a second `resumed` (mobile lifecycles).
    booted: bool,
}

impl<E: Embedder> FrogansApp<E> {
    /// A new app for `config`, driven through `embedder`. The engine is not
    /// touched until winit calls `resumed`.
    pub fn new(config: SessionConfig, embedder: E) -> Self {
        FrogansApp {
            config,
            embedder,
            session: None,
            booted: false,
        }
    }

    /// Map the engine's latest next-wake onto winit's control flow.
    fn apply_wake(&self, event_loop: &ActiveEventLoop) {
        let flow = match self.session.as_ref().map(Session::next_wake) {
            Some(NextWake::In(delay)) => {
                // A zero/near-zero delay means "call me again ASAP"; floor it so a
                // loading engine doesn't peg a core.
                let floor = std::time::Duration::from_millis(1);
                ControlFlow::WaitUntil(Instant::now() + delay.max(floor))
            }
            // Idle, or not booted yet: sleep until the next external event.
            Some(NextWake::Idle) | None => ControlFlow::Wait,
        };
        event_loop.set_control_flow(flow);
    }
}

impl<E: Embedder> ApplicationHandler for FrogansApp<E> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.booted {
            return;
        }
        self.booted = true;

        match Session::start(&self.config, &mut self.embedder, event_loop) {
            Ok(session) => self.session = Some(session),
            Err(e) => {
                eprintln!("frogans-surfaces: failed to start session: {e}");
                event_loop.exit();
                return;
            }
        }

        // Let the embedder drive the freshly-started session (e.g. an initial
        // leap), then pump whatever it queued.
        if let Some(session) = self.session.as_mut() {
            self.embedder.started(&mut Frogans::new(session));
            session.pump_pending(&mut self.embedder, event_loop);
        }

        self.apply_wake(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Close requests are reported to the engine inside `window_event` (per
        // window kind); the app exits only when the engine answers with `Stop`.
        if let Some(session) = self.session.as_mut() {
            session.window_event(window_id, event, &mut self.embedder, event_loop);
        }
        self.apply_wake(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(session) = self.session.as_mut() {
            session.pump_pending(&mut self.embedder, event_loop);
            if session.timeout_due() {
                session.pump_timeout(&mut self.embedder, event_loop);
            }
        }
        self.apply_wake(event_loop);
    }
}
