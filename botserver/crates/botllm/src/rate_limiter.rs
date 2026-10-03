// Rate limiter for LLM API calls
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;

/// Rate limits for an API provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Cerebras Inference free tier — the tightest published quota in the
    /// OpenAI-compatible tier, which is why Cerebras rows keep their own cap
    /// instead of inheriting the default.
    pub const fn cerebras_free_tier() -> Self {
        Self {
            requests_per_minute: 30,
            tokens_per_minute: 60_000,
            requests_per_day: 14_400,
            tokens_per_day: 1_000_000,
        }
    }

    /// Anthropic standard tier, applied to the native Claude client.
    pub const fn anthropic_standard() -> Self {
        Self {
            requests_per_minute: 50,
            tokens_per_minute: 40_000,
            requests_per_day: 10_000,
            tokens_per_day: 2_000_000,
        }
    }

    /// Azure OpenAI standard tier, applied to the Responses-API client.
    pub const fn azure_standard() -> Self {
        Self {
            requests_per_minute: 120,
            tokens_per_minute: 120_000,
            requests_per_day: 10_000,
            tokens_per_day: 10_000_000,
        }
    }

    /// z.ai / GLM standard tier.
    pub const fn glm_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 120_000,
            requests_per_day: 5_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// Amazon Bedrock on-demand quota, applied per model.
    pub const fn bedrock_standard() -> Self {
        Self {
            requests_per_minute: 100,
            tokens_per_minute: 200_000,
            requests_per_day: 20_000,
            tokens_per_day: 20_000_000,
        }
    }

    /// Google Vertex AI standard tier.
    pub const fn vertex_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 120_000,
            requests_per_day: 5_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// Kiro / CodeWhisperer, metered through the entitlement.
    pub const fn kiro_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 200_000,
            requests_per_day: 5_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// DeepInfra standard tier. DeepInfra publishes no hard cap, so this is an
    /// operating cap sized for sustained multi-bot traffic.
    pub const fn deepinfra_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 100_000,
            requests_per_day: 10_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// Fireworks AI standard tier.
    pub const fn fireworks_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 100_000,
            requests_per_day: 10_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// Together AI standard tier.
    pub const fn together_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 100_000,
            requests_per_day: 10_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// OpenRouter — the gateway publishes a credit-derived quota, so the caps
    /// stay conservative.
    pub const fn openrouter_standard() -> Self {
        Self {
            requests_per_minute: 20,
            tokens_per_minute: 50_000,
            requests_per_day: 1_000,
            tokens_per_day: 200_000,
        }
    }

    /// Requesty gateway, which meters the BYOK tier behind its own quota.
    pub const fn requesty_standard() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 100_000,
            requests_per_day: 10_000,
            tokens_per_day: 5_000_000,
        }
    }

    /// NVIDIA NIM, which meters per developer key.
    pub const fn nvidia_standard() -> Self {
        Self {
            requests_per_minute: 40,
            tokens_per_minute: 80_000,
            requests_per_day: 1_000,
            tokens_per_day: 500_000,
        }
    }

    /// Caps applied to an OpenAI-compatible host with no catalog row: generous
    /// enough for a self-hosted server, bounded enough that a mistyped URL
    /// cannot become an unbounded retry storm.
    pub const fn openai_compatible_default() -> Self {
        Self {
            requests_per_minute: 120,
            tokens_per_minute: 250_000,
            requests_per_day: 20_000,
            tokens_per_day: 20_000_000,
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
        // A limit of zero would leave the quota unrepresentable, so it becomes
        // one request per minute rather than an abort.
        let rpm_quota = NonZeroU32::new(limits.requests_per_minute)
            .unwrap_or(NonZeroU32::MIN)
            .max(NonZeroU32::MIN);
        let requests_per_minute = Arc::new(RateLimiter::direct(Quota::per_minute(rpm_quota)));

        // Tokens per minute (using semaphore as we need to track token count)
        let tokens_per_minute = Arc::new(Semaphore::new(
            limits.tokens_per_minute.try_into().unwrap_or(usize::MAX)
        ));

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
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
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        let reset_time = self.daily_request_reset.load(std::sync::atomic::Ordering::Relaxed);

        if now >= reset_time {
            // Reset counters
            self.daily_request_count.store(0, std::sync::atomic::Ordering::Relaxed);
            self.daily_token_count.store(0, std::sync::atomic::Ordering::Relaxed);

            // Set new reset time to tomorrow
            let tomorrow = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
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
