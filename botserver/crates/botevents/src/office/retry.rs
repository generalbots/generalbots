//! Retry policy for webhook delivery (#2503).
//!
//! Only transport and transient server failures are retried: 5xx, 429 and the
//! synthetic status 0 (network error). A 4xx is a permanent contract error and
//! goes straight to the dead-letter queue. Delays are computed deterministically
//! with capped exponential backoff; the delivery adapter applies them, which
//! keeps the policy unit-testable without sleeping.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(5, 500, 30_000, 2)
    }
}

impl RetryPolicy {
    #[must_use]
    pub fn new(max_attempts: u32, base_delay_ms: u64, max_delay_ms: u64, multiplier: u32) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_delay_ms: base_delay_ms.max(1),
            max_delay_ms: max_delay_ms.max(1),
            multiplier: multiplier.max(1),
        }
    }

    /// Whether a response can succeed on a later attempt.
    #[must_use]
    pub fn is_retryable(status_code: u16) -> bool {
        status_code == 0 || status_code == 429 || status_code >= 500
    }

    /// Delay before `attempt` (1-based). Attempt 1 is immediate.
    #[must_use]
    pub fn delay_for(&self, attempt: u32) -> u64 {
        if attempt <= 1 {
            return 0;
        }
        // Attempt 1 is the initial request (no delay); attempt 2 is the first
        // retry and waits one base interval, attempt 3 two, and so on.
        let exponent = attempt.saturating_sub(2);
        let factor = self.multiplier.saturating_pow(exponent.min(32));
        self.base_delay_ms
            .saturating_mul(u64::from(factor))
            .min(self.max_delay_ms)
    }

    /// Whether another attempt should be made after `attempt` saw `status`.
    #[must_use]
    pub fn should_retry(&self, attempt: u32, status: u16) -> bool {
        attempt < self.max_attempts && Self::is_retryable(status)
    }
}

#[cfg(test)]
mod tests {
    use super::RetryPolicy;

    #[test]
    fn capped_exponential_backoff() {
        let policy = RetryPolicy::new(6, 100, 1_000, 2);
        assert_eq!(policy.delay_for(1), 0);
        assert_eq!(policy.delay_for(2), 100);
        assert_eq!(policy.delay_for(3), 200);
        assert_eq!(policy.delay_for(4), 400);
        assert_eq!(policy.delay_for(5), 800);
        assert_eq!(policy.delay_for(6), 1_000);
        assert_eq!(policy.delay_for(20), 1_000);
    }

    #[test]
    fn only_transient_failures_are_retried() {
        assert!(RetryPolicy::is_retryable(503));
        assert!(RetryPolicy::is_retryable(500));
        assert!(RetryPolicy::is_retryable(429));
        assert!(RetryPolicy::is_retryable(0));
        assert!(!RetryPolicy::is_retryable(404));
        assert!(!RetryPolicy::is_retryable(401));
        assert!(!RetryPolicy::is_retryable(200));
    }

    #[test]
    fn retry_stops_at_the_policy_limit() {
        let policy = RetryPolicy::new(3, 10, 100, 2);
        assert!(policy.should_retry(1, 503));
        assert!(policy.should_retry(2, 503));
        assert!(!policy.should_retry(3, 503));
        assert!(!policy.should_retry(1, 400));
    }

    #[test]
    fn invalid_configuration_is_clamped() {
        let policy = RetryPolicy::new(0, 0, 0, 0);
        assert_eq!(policy.max_attempts, 1);
        assert_eq!(policy.multiplier, 1);
        assert!(!policy.should_retry(1, 503));
    }
}
