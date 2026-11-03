//! Run ownership (#1666).
//!
//! A chat turn no longer belongs to the socket that started it. The turn runs
//! as a task; frames go to the thread channel, and any socket subscribed to
//! that thread receives them. Closing the tab cannot cancel the work, and a
//! reconnect replays the persisted frames from the requested `seq`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::Value;
use tokio::sync::mpsc;

use botevents::RunState;

#[derive(Debug)]
struct RunMeta {
    run_id: String,
    state: RunState,
    cancelled: bool,
}

impl Default for RunMeta {
    fn default() -> Self {
        Self {
            run_id: String::new(),
            state: RunState::Succeeded,
            cancelled: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct ThreadChannel {
    meta: Mutex<RunMeta>,
    subscribers: Mutex<Vec<mpsc::UnboundedSender<Value>>>,
    last_seq: AtomicU64,
}

impl ThreadChannel {
    fn lock_meta(&self) -> std::sync::MutexGuard<'_, RunMeta> {
        match self.meta.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn lock_subscribers(&self) -> std::sync::MutexGuard<'_, Vec<mpsc::UnboundedSender<Value>>> {
        match self.subscribers.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn start(&self, run_id: &str) {
        let mut meta = self.lock_meta();
        meta.run_id = run_id.to_string();
        meta.state = RunState::Running;
        meta.cancelled = false;
    }

    #[must_use]
    pub fn current_run_id(&self) -> Option<String> {
        let meta = self.lock_meta();
        if meta.run_id.is_empty() {
            None
        } else {
            Some(meta.run_id.clone())
        }
    }

    #[must_use]
    pub fn state(&self) -> RunState {
        self.lock_meta().state
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        let meta = self.lock_meta();
        meta.state == RunState::Running && !meta.cancelled
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.lock_meta().cancelled
    }

    pub fn cancel(&self) {
        let mut meta = self.lock_meta();
        if !meta.state.is_terminal() {
            meta.cancelled = true;
            meta.state = RunState::Cancelling;
        }
    }

    pub fn finish(&self, state: RunState) {
        let mut meta = self.lock_meta();
        if meta.state.is_terminal() {
            return;
        }
        meta.state = if meta.cancelled {
            RunState::Cancelled
        } else {
            state
        };
    }

    pub fn set_state(&self, state: RunState) {
        let mut meta = self.lock_meta();
        if meta.state.is_terminal() {
            return;
        }
        if meta.state.can_transition_to(state) {
            meta.state = state;
        }
    }

    pub fn publish(&self, frame: Value) {
        if let Some(seq) = frame.get("seq").and_then(Value::as_u64) {
            self.last_seq.fetch_max(seq, Ordering::SeqCst);
        }
        let mut subscribers = self.lock_subscribers();
        subscribers.retain(|sender| sender.send(frame.clone()).is_ok());
    }

    /// Subscribes a socket. Returns the highest sequence already published so
    /// the caller can ask for the durable backlog instead of assuming it saw
    /// everything.
    pub fn subscribe(&self) -> (u64, mpsc::UnboundedReceiver<Value>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let last = self.last_seq.load(Ordering::SeqCst);
        self.lock_subscribers().push(sender);
        (last, receiver)
    }

    pub fn observe_seq(&self, seq: u64) {
        self.last_seq.fetch_max(seq, Ordering::SeqCst);
    }
}

#[derive(Debug, Default)]
pub struct RunBus {
    threads: Mutex<HashMap<String, Arc<ThreadChannel>>>,
}

impl RunBus {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn thread(&self, thread_id: &str) -> Arc<ThreadChannel> {
        let mut threads = match self.threads.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        threads
            .entry(thread_id.to_string())
            .or_insert_with(|| Arc::new(ThreadChannel::default()))
            .clone()
    }

    /// Marks a new run as the owner of the thread. A cancelled flag from a
    /// previous run is cleared; the sequence counter is not reset because
    /// ordering is thread-scoped.
    pub fn start(&self, thread_id: &str, run_id: &str) -> Arc<ThreadChannel> {
        let channel = self.thread(thread_id);
        channel.start(run_id);
        channel
    }

    #[must_use]
    pub fn current_run_id(&self, thread_id: &str) -> Option<String> {
        self.thread(thread_id).current_run_id()
    }

    pub fn publish(&self, thread_id: &str, frame: Value) {
        self.thread(thread_id).publish(frame);
    }

    pub fn subscribe(&self, thread_id: &str) -> (u64, mpsc::UnboundedReceiver<Value>) {
        self.thread(thread_id).subscribe()
    }

    pub fn cancel(&self, thread_id: &str) {
        self.thread(thread_id).cancel();
    }

    #[must_use]
    pub fn is_cancelled(&self, thread_id: &str) -> bool {
        self.thread(thread_id).is_cancelled()
    }

    #[must_use]
    pub fn is_running(&self, thread_id: &str) -> bool {
        self.thread(thread_id).is_running()
    }

    pub fn finish(&self, thread_id: &str, state: RunState) {
        self.thread(thread_id).finish(state);
    }
}

pub fn bus() -> &'static RunBus {
    static BUS: OnceLock<RunBus> = OnceLock::new();
    BUS.get_or_init(RunBus::new)
}

#[cfg(test)]
mod tests {
    use super::RunBus;
    use botevents::RunState;

    #[test]
    fn subscriber_receives_published_frames() {
        let bus = RunBus::new();
        let (_last, mut receiver) = bus.subscribe("thread-1");
        bus.publish("thread-1", serde_json::json!({ "seq": 1, "type": "RUN_STARTED" }));
        let frame = receiver.try_recv().expect("frame delivered");
        assert_eq!(frame["seq"], 1);
    }

    #[test]
    fn cancel_moves_running_to_cancelling_and_finish_latches_cancelled() {
        let bus = RunBus::new();
        bus.start("thread-1", "run-1");
        bus.cancel("thread-1");
        assert!(bus.is_cancelled("thread-1"));
        bus.finish("thread-1", RunState::Succeeded);
        assert_eq!(bus.thread("thread-1").state(), RunState::Cancelled);
    }

    #[test]
    fn a_terminal_run_cannot_restart_state() {
        let bus = RunBus::new();
        bus.start("thread-1", "run-1");
        bus.finish("thread-1", RunState::Succeeded);
        bus.thread("thread-1").set_state(RunState::Running);
        assert_eq!(bus.thread("thread-1").state(), RunState::Succeeded);
    }
}
