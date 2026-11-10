use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

pub type RetryPredicate = Arc<dyn Fn(&str) -> bool + Send + Sync>;

#[derive(Debug, Clone)]
pub enum ResilienceError {
    Timeout { duration: Duration },
    CircuitOpen { until: Option<Duration> },
    RetriesExhausted { attempts: u32, last_error: String },
    BulkheadFull { max_concurrent: usize },
    Operation(String),
}

impl std::fmt::Display for ResilienceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout { duration } => {
                write!(f, "Operation timed out after {:?}", duration)
            }
            Self::CircuitOpen { until } => {
                if let Some(d) = until {
                    write!(f, "Circuit breaker open, retry after {:?}", d)
                } else {
                    write!(f, "Circuit breaker open")
                }
            }
            Self::RetriesExhausted {
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "All {} retry attempts exhausted. Last error: {}",
                    attempts, last_error
                )
            }
            Self::BulkheadFull { max_concurrent } => {
                write!(
                    f,
                    "Bulkhead full, max {} concurrent requests",
                    max_concurrent
                )
            }
            Self::Operation(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for ResilienceError {}

#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
    retryable: Option<RetryPredicate>,
}

impl std::fmt::Debug for RetryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetryConfig")
            .field("max_attempts", &self.max_attempts)
            .field("initial_delay", &self.initial_delay)
            .field("max_delay", &self.max_delay)
            .field("backoff_multiplier", &self.backoff_multiplier)
            .field("jitter_factor", &self.jitter_factor)
            .field("retryable", &self.retryable.is_some())
            .finish()
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
            retryable: None,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with custom max attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts.max(1);
        self
    }

    /// Set initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay cap
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier.max(1.0);
        self
    }

    /// Set jitter factor (0.0 to 1.0)
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter_factor = jitter.clamp(0.0, 1.0);
        self
    }

    /// Set custom retryable predicate
    pub fn with_retryable<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.retryable = Some(Arc::new(predicate));
        self
    }

    /// Aggressive retry for critical operations
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            jitter_factor: 0.3,
            retryable: None,
        }
    }

    /// Conservative retry for non-critical operations
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retryable: None,
        }
    }

    fn calculate_delay(&self, attempt: u32) -> Duration {
        let exponent = i32::try_from(attempt.saturating_sub(1)).unwrap_or(0);
        let base_delay = self.backoff_multiplier.powi(exponent) * self.initial_delay.as_secs_f64();

        let capped_delay = base_delay.min(self.max_delay.as_secs_f64());

        let jitter = if self.jitter_factor > 0.0 {
            let jitter_range = capped_delay * self.jitter_factor;
            // Golden-ratio hashing spreads retries evenly across the jitter
            // window without drawing from a random source, which keeps the
            // backoff reproducible for a given attempt count.
            let pseudo_random = (f64::from(attempt) * 1.618_033_988_749_895_f64) % 1.0;
            (2.0_f64).mul_add(pseudo_random, -1.0) * jitter_range
        } else {
            0.0
        };

        Duration::from_secs_f64((capped_delay + jitter).max(0.001))
    }

    fn is_retryable(&self, error: &str) -> bool {
        if let Some(ref predicate) = self.retryable {
            predicate(error)
        } else {
            error.contains("timeout")
                || error.contains("connection")
                || error.contains("temporarily")
                || error.contains("503")
                || error.contains("429")
        }
    }
}

pub async fn retry<F, Fut, T>(config: &RetryConfig, mut operation: F) -> Result<T, ResilienceError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let mut last_error = String::new();

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == config.max_attempts {
                    last_error = e;
                    break;
                }

                if !config.is_retryable(&e) {
                    return Err(ResilienceError::Operation(e));
                }

                last_error = e;
                let delay = config.calculate_delay(attempt);
                sleep(delay).await;
            }
        }
    }

    Err(ResilienceError::RetriesExhausted {
        attempts: config.max_attempts,
        last_error,
    })
}

pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, ResilienceError>
where
    F: Future<Output = T>,
{
    timeout(duration, future)
        .await
        .map_err(|_| ResilienceError::Timeout { duration })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn fast_config(attempts: u32) -> RetryConfig {
        RetryConfig {
            max_attempts: attempts,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
            retryable: None,
        }
    }

    #[test]
    fn default_is_retryable_only_for_transient_errors() {
        let config = fast_config(3);
        assert!(config.is_retryable("connection reset"));
        assert!(config.is_retryable("timeout after 5s"));
        assert!(config.is_retryable("server said 503"));
        assert!(config.is_retryable("too many requests: 429"));
        assert!(!config.is_retryable("invalid argument"));
    }

    #[test]
    fn custom_predicate_overrides_the_defaults() {
        let config = fast_config(2).with_retryable(|error| error == "again");
        assert!(config.is_retryable("again"));
        assert!(!config.is_retryable("timeout"));
    }

    #[test]
    fn delay_grows_and_is_capped() {
        let growing = fast_config(10)
            .with_initial_delay(Duration::from_millis(10))
            .with_max_delay(Duration::from_secs(1));
        let first = growing.calculate_delay(1);
        let second = growing.calculate_delay(2);
        assert!(second > first, "{second:?} must exceed {first:?}");

        let capped = fast_config(10)
            .with_initial_delay(Duration::from_millis(10))
            .with_max_delay(Duration::from_millis(5));
        assert!(capped.calculate_delay(10) <= Duration::from_millis(5));
    }

    #[tokio::test]
    async fn retry_returns_after_a_transient_failure() {
        let attempts = AtomicU32::new(0);
        let result = retry(&fast_config(3), || {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                if attempt < 2 {
                    Err("connection reset".to_string())
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.expect("third attempt succeeds"), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn retry_stops_immediately_on_a_permanent_error() {
        let attempts = AtomicU32::new(0);
        let result: Result<(), ResilienceError> = retry(&fast_config(5), || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err("invalid argument".to_string()) }
        })
        .await;
        match result {
            Err(ResilienceError::Operation(message)) => assert_eq!(message, "invalid argument"),
            other => panic!("expected Operation, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_reports_exhaustion_after_the_last_attempt() {
        let result: Result<(), ResilienceError> =
            retry(&fast_config(2), || async { Err("timeout".to_string()) }).await;
        match result {
            Err(ResilienceError::RetriesExhausted { attempts, last_error }) => {
                assert_eq!(attempts, 2);
                assert_eq!(last_error, "timeout");
            }
            other => panic!("expected RetriesExhausted, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn with_timeout_reports_the_elapsed_budget() {
        let result = with_timeout(Duration::from_millis(1), async {
            sleep(Duration::from_millis(50)).await;
        })
        .await;
        match result {
            Err(ResilienceError::Timeout { duration }) => {
                assert_eq!(duration, Duration::from_millis(1));
            }
            other => panic!("expected Timeout, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn with_timeout_returns_the_value_when_fast_enough() {
        let value = with_timeout(Duration::from_secs(1), async { 7 }).await;
        assert_eq!(value.expect("in time"), 7);
    }
}
