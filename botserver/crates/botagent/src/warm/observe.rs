//! Timing of one bridged turn, for the log: how long the process took to be
//! ready and how long after the prompt the first thought or text came; and
//! whether anything of the turn reached the chat yet (a turn that did can no
//! longer be retried on a fresh process).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

pub(crate) struct TurnClock {
    engine: &'static str,
    origin: &'static str,
    started: Instant,
    prompted: std::sync::Mutex<Option<Instant>>,
    first: AtomicBool,
    touched: AtomicBool,
}

impl TurnClock {
    pub(crate) fn start(engine: &'static str, warm: bool) -> Self {
        Self {
            engine,
            origin: if warm { "warm" } else { "fresh" },
            started: Instant::now(),
            prompted: std::sync::Mutex::new(None),
            first: AtomicBool::new(false),
            touched: AtomicBool::new(false),
        }
    }

    /// The process answered its handshake.
    pub(crate) fn ready(&self) {
        tracing::info!(
            "{} bridge: process ready after {} ms ({})",
            self.engine,
            self.started.elapsed().as_millis(),
            self.origin
        );
    }

    /// The prompt was sent.
    pub(crate) fn prompted(&self) {
        if let Ok(mut at) = self.prompted.lock() {
            *at = Some(Instant::now());
        }
        tracing::info!(
            "{} bridge: prompt sent {} ms after the turn began ({})",
            self.engine,
            self.started.elapsed().as_millis(),
            self.origin
        );
    }

    /// Something of the turn reached the chat: a thought, text or a step.
    pub(crate) fn touch(&self, kind: &str) {
        self.touched.store(true, Ordering::SeqCst);
        if self.first.swap(true, Ordering::SeqCst) {
            return;
        }
        let since_prompt = self
            .prompted
            .lock()
            .ok()
            .and_then(|at| *at)
            .map_or_else(|| "-".to_string(), |at| at.elapsed().as_millis().to_string());
        tracing::info!(
            "{} bridge: first {kind} {since_prompt} ms after the prompt, {} ms after the turn began ({})",
            self.engine,
            self.started.elapsed().as_millis(),
            self.origin
        );
    }

    pub(crate) fn touched(&self) -> bool {
        self.touched.load(Ordering::SeqCst)
    }
}
