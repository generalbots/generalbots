// Rate limiter for LLM API calls
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;

/// Rate limits for an API provider
#[derive(Debug, Clone, Copy)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub requests_per_day: u32,
    pub tokens_per_day: u32,
}

impl RateLimits {
    /// Groq free tier rate limits
    pub const fn groq_free_tier() -> Self {
        Self {
            requests_per_minute: 30,
            tokens_per_minute: 8_000,
            requests_per_day: 1_000,
            tokens_per_day: 200_000,
        }
    }

    /// OpenAI free tier rate limits
    pub const fn openai_free_tier() -> Self {
        Self {
            requests_per_minute: 3,
            tokens_per_minute: 40_000,
            requests_per_day: 200,
            tokens_per_day: 150_000,
        }
    }

    /// No rate limiting (for local models)
    pub const fn unlimited() -> Self {
        Self {
            requests_per_minute: u32::MAX,
            tokens_per_minute: u32::MAX,
            requests_per_day: u32::MAX,
            tokens_per_day: u32::MAX,
        }
    }
}

/// A rate limiter for API requests
pub struct ApiRateLimiter {
    requests_per_minute: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    tokens_per_minute: Arc<Semaphore>,
    // Track daily request count with a simple counter and reset time
    daily_request_count: Arc<std::sync::atomic::AtomicU32>,
    daily_request_reset: Arc<std::sync::atomic::AtomicU64>,
    daily_token_count: Arc<std::sync::atomic::AtomicU32>,
    daily_token_reset: Arc<std::sync::atomic::AtomicU64>,
    requests_per_day: u32,
    tokens_per_day: u32,
}

impl std::fmt::Debug for ApiRateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiRateLimiter")
            .field("requests_per_minute", &self.requests_per_minute)
            .field("tokens_per_minute", &"Semaphore")
            .field("daily_request_count", &self.daily_request_count)
            .field("daily_token_count", &self.daily_token_count)
            .field("requests_per_day", &self.requests_per_day)
            .field("tokens_per_day", &self.tokens_per_day)
            .finish()
    }
}

impl Clone for ApiRateLimiter {
    fn clone(&self) -> Self {
        Self {
            requests_per_minute: Arc::clone(&self.requests_per_minute),
            tokens_per_minute: Arc::clone(&self.tokens_per_minute),
            daily_request_count: Arc::clone(&self.daily_request_count),
            daily_request_reset: Arc::clone(&self.daily_request_reset),
            daily_token_count: Arc::clone(&self.daily_token_count),
            daily_token_reset: Arc::clone(&self.daily_token_reset),
            requests_per_day: self.requests_per_day,
            tokens_per_day: self.tokens_per_day,
        }
    }
}

impl ApiRateLimiter {
    /// Create a new rate limiter with the specified limits
    pub fn new(limits: RateLimits) -> Self {
        // Requests per minute limiter
        let rpm_quota = Quota::per_minute(NonZeroU32::new(limits.requests_per_minute).unwrap());
        let requests_per_minute = Arc::new(RateLimiter::direct(rpm_quota));

        // Tokens per minute (using semaphore as we need to track token count)
        let tokens_per_minute = Arc::new(Semaphore::new(
            limits.tokens_per_minute.try_into().unwrap_or(usize::MAX)
        ));

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let tomorrow = now + 86400;

        Self {
            requests_per_minute,
            tokens_per_minute,
            daily_request_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            daily_request_reset: Arc::new(std::sync::atomic::AtomicU64::new(tomorrow)),
            daily_token_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            daily_token_reset: Arc::new(std::sync::atomic::AtomicU64::new(tomorrow)),
            requests_per_day: limits.requests_per_day,
            tokens_per_day: limits.tokens_per_day,
        }
    }

    /// Create an unlimited rate limiter (for local models)
    pub fn unlimited() -> Self {
        Self::new(RateLimits::unlimited())
    }

    /// Check if daily limits need resetting and reset if needed
    fn check_and_reset_daily(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let reset_time = self.daily_request_reset.load(std::sync::atomic::Ordering::Relaxed);

        if now >= reset_time {
            // Reset counters
            self.daily_request_count.store(0, std::sync::atomic::Ordering::Relaxed);
            self.daily_token_count.store(0, std::sync::atomic::Ordering::Relaxed);

            // Set new reset time to tomorrow
            let tomorrow = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + 86400;
            self.daily_request_reset.store(tomorrow, std::sync::atomic::Ordering::Relaxed);
            self.daily_token_reset.store(tomorrow, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Acquire permission for a request with estimated token count
    /// Returns when the request can proceed
    pub async fn acquire(&self, estimated_tokens: usize) -> Result<(), RateLimitError> {
        // Check and reset daily limits if needed
        self.check_and_reset_daily();

        // Check request rate limits
        self.requests_per_minute.until_ready().await;

        // Check daily request limit
        let current_requests = self.daily_request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if current_requests >= self.requests_per_day {
            self.daily_request_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            return Err(RateLimitError::DailyRateLimitExceeded);
        }

        // Check token rate limits
        let tokens_to_acquire = (estimated_tokens.min(8_000) as u32) as usize;

        // Try to acquire token permits for minute limit
        let tpm_available = self.tokens_per_minute.available_permits();
        if tpm_available < tokens_to_acquire {
            return Err(RateLimitError::TokenRateLimitExceeded);
        }

        // Check daily token limit
        let current_tokens = self.daily_token_count.fetch_add(tokens_to_acquire as u32, std::sync::atomic::Ordering::Relaxed);
        if current_tokens + (tokens_to_acquire as u32) > self.tokens_per_day {
            self.daily_token_count.fetch_sub(tokens_to_acquire as u32, std::sync::atomic::Ordering::Relaxed);
            return Err(RateLimitError::DailyTokenLimitExceeded);
        }

        // Acquire the permits (this will wait if needed)
        let semaphore = Arc::clone(&self.tokens_per_minute);
        let _permits = semaphore.acquire_many_owned(tokens_to_acquire as u32).await;
        // Permits are held until the request completes

        Ok(())
    }

    /// Release token permits after request completes
    pub fn release_tokens(&self, _tokens: u32) {
        // Note: We don't release the daily token count as it's already "used"
        // But we do need to release the semaphore permits for the minute limit
        // The permits will be automatically released when dropped
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    RateLimitExceeded,
    DailyRateLimitExceeded,
    TokenRateLimitExceeded,
    DailyTokenLimitExceeded,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            RateLimitError::DailyRateLimitExceeded => write!(f, "Daily request limit exceeded"),
            RateLimitError::TokenRateLimitExceeded => write!(f, "Token per minute limit exceeded"),
            RateLimitError::DailyTokenLimitExceeded => write!(f, "Daily token limit exceeded"),
        }
    }
}

impl std::error::Error for RateLimitError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limits_display() {
        let limits = RateLimits::groq_free_tier();
        assert_eq!(limits.requests_per_minute, 30);
        assert_eq!(limits.tokens_per_minute, 8_000);
        assert_eq!(limits.requests_per_day, 1_000);
        assert_eq!(limits.tokens_per_day, 200_000);
    }

    #[test]
    fn test_unlimited_limits() {
        let limits = RateLimits::unlimited();
        assert_eq!(limits.requests_per_minute, u32::MAX);
    }
}
