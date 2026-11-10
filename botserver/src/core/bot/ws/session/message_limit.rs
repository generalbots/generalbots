//! IMP-07 — inbound WebSocket message rate limit.
//!
//! The HTTP rate limiter never sees a WebSocket frame: once the socket is
//! upgraded, one connection can push an unbounded stream of turns, each of
//! which costs an LLM call. AGENTS.md asks for ~10 messages per second per
//! client, so every socket owns a token bucket and the loop drops the frames
//! above it instead of feeding the model.
//!
//! The bucket is per connection, not per session: a reconnect starts with a
//! fresh budget, which is the same trade-off a browser makes for a reload and
//! keeps the limiter free of shared state and cleanup.

use std::num::NonZeroU32;

use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};

/// Sustained inbound frames per second allowed on one socket.
pub(super) const MESSAGES_PER_SECOND: u32 = 10;
/// How much idle time a client may bank before the limit applies.
pub(super) const BURST: u32 = 20;

type DirectLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

pub(super) struct MessageLimiter {
    limiter: DirectLimiter,
}

impl MessageLimiter {
    pub(super) fn new() -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(MESSAGES_PER_SECOND).unwrap_or(NonZeroU32::MIN),
        )
        .allow_burst(NonZeroU32::new(BURST).unwrap_or(NonZeroU32::MIN));
        Self {
            limiter: RateLimiter::direct(quota),
        }
    }

    /// `true` when this frame may be processed, `false` when it must be
    /// dropped with a `rate_limited` frame.
    pub(super) fn allow(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageLimiter, BURST};

    #[test]
    fn a_burst_is_allowed_and_the_frame_after_it_is_not() {
        let limiter = MessageLimiter::new();
        for _ in 0..BURST {
            assert!(limiter.allow(), "the burst budget must be usable");
        }
        assert!(!limiter.allow(), "the frame past the burst must be dropped");
    }

    #[test]
    fn the_sustained_rate_is_ten_messages_per_second() {
        assert_eq!(super::MESSAGES_PER_SECOND, 10);
    }
}
