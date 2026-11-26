//! Watchdog for the stretch of a chat turn before the model call.
//!
//! A turn that stopped between the inbound frame and the model call left the
//! client waiting with no frame and the log with no line naming the step. Each
//! external wait in that stretch is bounded with [`bounded`], each stage logs
//! one line through [`stage`], and [`TurnWatch::run`] ends a turn that made no
//! progress (no frame, no new stage) for the stage's limit with a classified
//! `timeout` error frame, logging the stage it stopped at. Once the turn
//! reaches the model call ([`reached_model`]) the model path's own timeout and
//! keep-alive frames take over and the watchdog stands down.

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use botlib::models::BotResponse;
use uuid::Uuid;

use super::llm::error_classify::LlmErrorCode;
use super::sink::ChannelSink;
use super::types::{PipelineResult, SessionIdentity};

/// Longest a pre-model stage may go without progress. The assistant shell
/// gives up 90 s after the last frame; the server ends the turn first so the
/// log names the stage.
pub const STAGE_LIMIT: Duration = Duration::from_secs(75);
/// Limit for a stage that legitimately runs long without frames (the first
/// workspace provisioning downloads an image).
pub const LONG_STAGE_LIMIT: Duration = Duration::from_secs(600);
/// Bound for one wait on a shared lock before the model call.
pub const STEP_LIMIT: Duration = Duration::from_secs(10);
/// Bound for one Redis round trip before the model call.
pub const REDIS_STEP_LIMIT: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
struct Progress {
    stage: &'static str,
    limit: Duration,
    stage_at: Instant,
    last_activity: Instant,
    reached_model: bool,
}

impl Progress {
    fn new(now: Instant) -> Self {
        Self {
            stage: "received",
            limit: STAGE_LIMIT,
            stage_at: now,
            last_activity: now,
            reached_model: false,
        }
    }

    /// Enters a stage and returns the time spent in the previous one.
    fn enter(&mut self, stage: &'static str, limit: Duration, now: Instant) -> Duration {
        let spent = now.saturating_duration_since(self.stage_at);
        self.stage = stage;
        self.limit = limit;
        self.stage_at = now;
        self.last_activity = now;
        spent
    }

    fn frame(&mut self, now: Instant) {
        self.last_activity = now;
    }

    /// The stage and its quiet time when the turn is stuck before the model.
    fn stalled(&self, now: Instant) -> Option<(&'static str, Duration)> {
        if self.reached_model {
            return None;
        }
        let quiet = now.saturating_duration_since(self.last_activity);
        (quiet >= self.limit).then_some((self.stage, quiet))
    }
}

#[derive(Clone)]
struct Shared {
    session_id: Uuid,
    progress: Arc<Mutex<Progress>>,
}

impl Shared {
    fn update<R>(&self, change: impl FnOnce(&mut Progress) -> R) -> Option<R> {
        self.progress.lock().ok().map(|mut progress| change(&mut *progress))
    }
}

tokio::task_local! {
    static CURRENT: Shared;
}

fn enter_stage(name: &'static str, limit: Duration) {
    let logged = CURRENT.try_with(|shared| {
        let spent = shared.update(|progress| progress.enter(name, limit, Instant::now()));
        let spent_ms = spent.unwrap_or_else(|| { log::debug!("using default for stage progress"); Default::default() }).as_millis();
        let session_id = shared.session_id;
        botlog::safe_info!("turn stage: {name} session={session_id} previous_ms={spent_ms}");
    });
    if logged.is_err() {
        log::debug!("turn stage: {name}");
    }
}

/// Records and logs the stage the current turn has reached.
pub fn stage(name: &'static str) {
    enter_stage(name, STAGE_LIMIT);
}

/// Like [`stage`], for a stage allowed [`LONG_STAGE_LIMIT`].
pub fn long_stage(name: &'static str) {
    enter_stage(name, LONG_STAGE_LIMIT);
}

/// The turn reached the model call: the watchdog stands down.
pub fn reached_model() {
    stage("model call");
    let marked = CURRENT.try_with(|shared| shared.update(|progress| progress.reached_model = true));
    if marked.is_err() {
        log::debug!("model call outside a watched turn");
    }
}

/// Awaits one external step for at most `limit`; `None` (and a warning naming
/// the step) when it did not finish in time.
pub async fn bounded<F: Future>(step: &'static str, limit: Duration, wait: F) -> Option<F::Output> {
    match tokio::time::timeout(limit, wait).await {
        Ok(output) => Some(output),
        Err(_) => {
            log::warn!("turn step timed out: step={step} limit_ms={}", limit.as_millis());
            None
        }
    }
}

/// One watchdog per WebSocket run task, reused for each queued turn.
pub struct TurnWatch {
    shared: Shared,
}

impl TurnWatch {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            shared: Shared {
                session_id,
                progress: Arc::new(Mutex::new(Progress::new(Instant::now()))),
            },
        }
    }

    /// Wraps the channel so every frame the turn sends counts as progress.
    pub fn sink<'a>(&self, inner: &'a dyn ChannelSink) -> WatchedSink<'a> {
        WatchedSink {
            inner,
            shared: self.shared.clone(),
        }
    }

    /// Runs one turn. `None` when it stalled before the model call: the turn
    /// was dropped and the client received a `timeout` error frame.
    pub async fn run<F: Future>(&self, identity: &SessionIdentity<'_>, turn: F) -> Option<F::Output> {
        self.shared.update(|progress| *progress = Progress::new(Instant::now()));
        let mut watched = Box::pin(CURRENT.scope(self.shared.clone(), turn));
        let mut ticker = tokio::time::interval(POLL_INTERVAL);
        let (stage, quiet) = loop {
            tokio::select! {
                output = &mut watched => return Some(output),
                _ = ticker.tick() => {
                    let now = Instant::now();
                    if let Some(stall) = self.shared.update(|progress| progress.stalled(now)).flatten() {
                        break stall;
                    }
                }
            }
        };
        drop(watched);
        let session_id = self.shared.session_id;
        botlog::safe_warn!(
            "turn stalled before the model call: stage={stage} quiet_s={} session={session_id}",
            quiet.as_secs()
        );
        super::llm::stream_errors::send_classified_error(identity.sink, identity, LlmErrorCode::Timeout)
            .await;
        None
    }
}

/// The run task's channel, counting each frame as turn progress.
pub struct WatchedSink<'a> {
    inner: &'a dyn ChannelSink,
    shared: Shared,
}

impl WatchedSink<'_> {
    fn sent(&self) {
        self.shared.update(|progress| progress.frame(Instant::now()));
    }
}

#[async_trait]
impl ChannelSink for WatchedSink<'_> {
    async fn send_bot_response(&self, response: &BotResponse) -> PipelineResult<()> {
        let result = self.inner.send_bot_response(response).await;
        self.sent();
        result
    }

    async fn send_raw_json(&self, json: &serde_json::Value) -> PipelineResult<()> {
        let result = self.inner.send_raw_json(json).await;
        self.sent();
        result
    }

    async fn send_error(&self, session_id: &str, message: &str) -> PipelineResult<()> {
        let result = self.inner.send_error(session_id, message).await;
        self.sent();
        result
    }

    fn channel_type(&self) -> &str {
        self.inner.channel_type()
    }

    fn supports_streaming(&self) -> bool {
        self.inner.supports_streaming()
    }

    fn supports_suggestions(&self) -> bool {
        self.inner.supports_suggestions()
    }

    fn supports_raw_frames(&self) -> bool {
        self.inner.supports_raw_frames()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_turn_is_not_stalled_before_the_limit() {
        let start = Instant::now();
        let progress = Progress::new(start);
        assert!(progress.stalled(start + STAGE_LIMIT - Duration::from_secs(1)).is_none());
    }

    #[test]
    fn a_quiet_turn_is_stalled_at_the_stage_it_stopped() {
        let start = Instant::now();
        let mut progress = Progress::new(start);
        progress.enter("agent switch resolved", STAGE_LIMIT, start);
        let stall = progress.stalled(start + STAGE_LIMIT);
        assert_eq!(stall.map(|(stage, _)| stage), Some("agent switch resolved"));
    }

    #[test]
    fn a_frame_or_a_new_stage_counts_as_progress() {
        let start = Instant::now();
        let mut progress = Progress::new(start);
        progress.frame(start + Duration::from_secs(60));
        assert!(progress.stalled(start + STAGE_LIMIT).is_none());
        let spent = progress.enter("routing", STAGE_LIMIT, start + Duration::from_secs(70));
        assert_eq!(spent, Duration::from_secs(70));
        assert!(progress.stalled(start + Duration::from_secs(140)).is_none());
    }

    #[test]
    fn a_long_stage_gets_the_long_limit() {
        let start = Instant::now();
        let mut progress = Progress::new(start);
        progress.enter("workspace", LONG_STAGE_LIMIT, start);
        assert!(progress.stalled(start + STAGE_LIMIT * 2).is_none());
        assert!(progress.stalled(start + LONG_STAGE_LIMIT).is_some());
    }

    #[test]
    fn the_watchdog_stands_down_at_the_model_call() {
        let start = Instant::now();
        let mut progress = Progress::new(start);
        progress.reached_model = true;
        assert!(progress.stalled(start + LONG_STAGE_LIMIT * 2).is_none());
    }

    #[tokio::test]
    async fn bounded_names_a_step_that_never_finishes() {
        let stuck = bounded("never", Duration::from_millis(10), std::future::pending::<()>()).await;
        assert!(stuck.is_none());
        let done = bounded("ready", Duration::from_secs(1), async { 7 }).await;
        assert_eq!(done, Some(7));
    }

    #[tokio::test]
    async fn stages_outside_a_watched_turn_are_harmless() {
        stage("routing");
        reached_model();
        let inside = CURRENT
            .scope(
                Shared {
                    session_id: Uuid::nil(),
                    progress: Arc::new(Mutex::new(Progress::new(Instant::now()))),
                },
                async {
                    reached_model();
                    CURRENT.with(|shared| shared.update(|progress| progress.reached_model))
                },
            )
            .await;
        assert_eq!(inside, Some(true));
    }
}
