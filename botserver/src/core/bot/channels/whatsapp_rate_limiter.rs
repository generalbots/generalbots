//! WhatsApp Rate Limiter
//!
//! Implements rate limiting for WhatsApp Cloud API based on Meta's throughput tiers.
//!
//! ## Meta WhatsApp Rate Limits (per phone number)
//!
//! | Tier | Messages/second | Conversations/day |
//! |------|-----------------|-------------------|
//! | 1    | 40              | 1,000             |
//! | 2    | 80              | 10,000            |
//! | 3    | 200             | 100,000           |
//! | 4    | 400+            | Unlimited         |
//!
//! New phone numbers start at Tier 1.

use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

/// WhatsApp throughput tier levels (matches Meta's tiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum WhatsAppTier {
    /// Tier 1: New phone numbers (40 msg/s, 1000 conv/day)
    #[default]
    Tier1,
    /// Tier 2: Medium quality (80 msg/s, 10000 conv/day)
    Tier2,
    /// Tier 3: High quality (200 msg/s, 100000 conv/day)
    Tier3,
    /// Tier 4: Premium (400+ msg/s, unlimited)
    Tier4,
}


impl WhatsAppTier {
    /// Get messages per second for this tier
    pub fn messages_per_second(&self) -> u32 {
        match self {
            Self::Tier1 => 40,
            Self::Tier2 => 80,
            Self::Tier3 => 200,
            Self::Tier4 => 400,
        }
    }

    /// Get burst size (slightly higher to allow brief spikes)
    pub fn burst_size(&self) -> u32 {
        match self {
            Self::Tier1 => 50,
            Self::Tier2 => 100,
            Self::Tier3 => 250,
            Self::Tier4 => 500,
        }
    }

    /// Get minimum delay between messages (for streaming)
    pub fn min_delay_ms(&self) -> u64 {
        match self {
            Self::Tier1 => 25,  // 40 msg/s = 25ms between messages
            Self::Tier2 => 12,  // 80 msg/s = 12.5ms
            Self::Tier3 => 5,   // 200 msg/s = 5ms
            Self::Tier4 => 2,   // 400 msg/s = 2.5ms
        }
    }
}

impl std::fmt::Display for WhatsAppTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tier1 => write!(f, "Tier 1 (40 msg/s)"),
            Self::Tier2 => write!(f, "Tier 2 (80 msg/s)"),
            Self::Tier3 => write!(f, "Tier 3 (200 msg/s)"),
            Self::Tier4 => write!(f, "Tier 4 (400+ msg/s)"),
        }
    }
}

/// Configuration for WhatsApp rate limiting
#[derive(Debug, Clone)]
pub struct WhatsAppRateLimitConfig {
    /// Throughput tier (determines rate limits)
    pub tier: WhatsAppTier,
    /// Custom messages per second (overrides tier if set)
    pub custom_mps: Option<u32>,
    /// Custom burst size (overrides tier if set)
    pub custom_burst: Option<u32>,
    /// Enable rate limiting
    pub enabled: bool,
}

impl Default for WhatsAppRateLimitConfig {
    fn default() -> Self {
        Self {
            tier: WhatsAppTier::Tier1,
            custom_mps: None,
            custom_burst: None,
            enabled: true,
        }
    }
}

impl WhatsAppRateLimitConfig {
    /// Create config for a specific tier
    pub fn from_tier(tier: WhatsAppTier) -> Self {
        Self {
            tier,
            ..Default::default()
        }
    }

    /// Create config with custom rate
    pub fn custom(messages_per_second: u32, burst_size: u32) -> Self {
        Self {
            tier: WhatsAppTier::Tier1,
            custom_mps: Some(messages_per_second),
            custom_burst: Some(burst_size),
            enabled: true,
        }
    }

    /// Get effective messages per second
    pub fn effective_mps(&self) -> u32 {
        self.custom_mps.unwrap_or_else(|| self.tier.messages_per_second())
    }

    /// Get effective burst size
    pub fn effective_burst(&self) -> u32 {
        self.custom_burst.unwrap_or_else(|| self.tier.burst_size())
    }
}

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// WhatsApp Rate Limiter
///
/// Uses token bucket algorithm via governor crate.
/// Thread-safe and async-friendly.
/// Implements per-recipient rate limiting (1 msg/sec per phone number).
#[derive(Debug)]
pub struct WhatsAppRateLimiter {
    limiter: Arc<Limiter>,
    config: WhatsAppRateLimitConfig,
    min_delay: Duration,
    per_recipient_limiters: Arc<Mutex<HashMap<String, Arc<Limiter>>>>,
}

impl WhatsAppRateLimiter {
    /// Create a new rate limiter with default Tier 1 settings
    pub fn new() -> Self {
        Self::with_config(WhatsAppRateLimitConfig::default())
    }

    /// Create a rate limiter for a specific tier
    pub fn from_tier(tier: WhatsAppTier) -> Self {
        Self::with_config(WhatsAppRateLimitConfig::from_tier(tier))
    }

    /// Create a rate limiter with custom configuration
    pub fn with_config(config: WhatsAppRateLimitConfig) -> Self {
        let mps = config.effective_mps();
        let burst = config.effective_burst();
        let min_delay = Duration::from_millis(config.tier.min_delay_ms());

        let quota = Quota::per_second(
            NonZeroU32::new(mps).unwrap_or(NonZeroU32::MIN)
        )
        .allow_burst(
            NonZeroU32::new(burst).unwrap_or(NonZeroU32::MIN)
        );

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
            config,
            min_delay,
            per_recipient_limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a message can be sent immediately
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }

    /// Wait until a message can be sent (async)
    ///
    /// This will block until the rate limiter allows the message.
    /// Uses exponential backoff for waiting.
    pub async fn acquire(&self) {
        if !self.config.enabled {
            return;
        }

        // Try to acquire immediately
        if self.limiter.check().is_ok() {
            return;
        }

        // If not available, wait with minimum delay
        loop {
            sleep(self.min_delay).await;
            if self.limiter.check().is_ok() {
                return;
            }
        }
    }

    /// Wait until a message can be sent to a specific recipient (async)
    ///
    /// Enforces 1 message per second per phone number (Meta requirement).
    pub async fn acquire_for_recipient(&self, phone_number: &str) {
        if !self.config.enabled {
            return;
        }

        // Get or create per-recipient limiter (1 msg/sec)
        let recipient_limiter = {
            let mut limiters = self.per_recipient_limiters.lock().await;
            limiters
                .entry(phone_number.to_string())
                .or_insert_with(|| {
                    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
                    Arc::new(RateLimiter::direct(quota))
                })
                .clone()
        };

        // Wait for recipient-specific rate limit
        loop {
            if recipient_limiter.check().is_ok() {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        // Also wait for global rate limit
        self.acquire().await;
    }

    /// Try to acquire with timeout
    ///
    /// Returns true if acquired, false if timed out
    pub async fn try_acquire_timeout(&self, timeout: Duration) -> bool {
        if !self.config.enabled {
            return true;
        }

        if self.limiter.check().is_ok() {
            return true;
        }

        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            sleep(self.min_delay).await;
            if self.limiter.check().is_ok() {
                return true;
            }
        }
        false
    }

    /// Get current configuration
    pub fn config(&self) -> &WhatsAppRateLimitConfig {
        &self.config
    }

    /// Get the tier
    pub fn tier(&self) -> WhatsAppTier {
        self.config.tier
    }

    /// Get minimum delay between messages
    pub fn min_delay(&self) -> Duration {
        self.min_delay
    }
}

impl Default for WhatsAppRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WhatsAppRateLimiter {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
            config: self.config.clone(),
            min_delay: self.min_delay,
            per_recipient_limiters: Arc::clone(&self.per_recipient_limiters),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_defaults() {
        assert_eq!(WhatsAppTier::Tier1.messages_per_second(), 40);
        assert_eq!(WhatsAppTier::Tier2.messages_per_second(), 80);
        assert_eq!(WhatsAppTier::Tier3.messages_per_second(), 200);
        assert_eq!(WhatsAppTier::Tier4.messages_per_second(), 400);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = WhatsAppRateLimiter::new();
        assert!(limiter.check());
    }

    #[test]
    fn test_tier_limiter() {
        let limiter = WhatsAppRateLimiter::from_tier(WhatsAppTier::Tier2);
        assert_eq!(limiter.tier(), WhatsAppTier::Tier2);
        assert!(limiter.check());
    }

    #[test]
    fn test_custom_config() {
        let config = WhatsAppRateLimitConfig::custom(100, 150);
        assert_eq!(config.effective_mps(), 100);
        assert_eq!(config.effective_burst(), 150);
    }

    #[tokio::test]
    async fn test_acquire() {
        let limiter = WhatsAppRateLimiter::from_tier(WhatsAppTier::Tier4);
        // Should acquire immediately
        limiter.acquire().await;
    }

    #[tokio::test]
    async fn test_try_acquire_timeout() {
        let limiter = WhatsAppRateLimiter::new();
        // Should succeed immediately
        let result = limiter.try_acquire_timeout(Duration::from_millis(100)).await;
        assert!(result);
    }
}
