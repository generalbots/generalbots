//! Per-run spend caps (#1696).
//!
//! A chat run gets a budget before the first token: a hard token limit and a
//! wall-clock limit (which also bounds the VM/terminal work a turn can do).
//! Caps are opt-in: both zero means unlimited, which keeps existing
//! deployments unchanged. Values come from the environment so they can be set
//! per deployment/plan today:
//!
//! - `GB_RUN_MAX_TOKENS` — approximate tokens for the whole turn (chars / 4);
//! - `GB_RUN_MAX_SECONDS` — wall clock from dispatch to the latest chunk.
//!
//! Enforcement lives in the websocket stream loop: when `check()` reports a
//! breach the turn stops consuming the model stream and returns an explicit
//! "run stopped" answer instead of silently continuing to spend.

use std::time::{Duration, Instant};

/// Limits for one run. Zero disables that limit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RunCaps {
    pub max_tokens: u64,
    pub max_seconds: u64,
}

impl RunCaps {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            max_tokens: env_u64("GB_RUN_MAX_TOKENS"),
            max_seconds: env_u64("GB_RUN_MAX_SECONDS"),
        }
    }

    /// Per-bot (plan/workspace) settings override the deployment defaults.
    /// Empty, missing or unparsable bot values keep the environment value, so
    /// an existing deployment is unchanged until a limit is configured.
    #[must_use]
    pub fn merge(bot_max_tokens: Option<&str>, bot_max_seconds: Option<&str>, defaults: Self) -> Self {
        Self {
            max_tokens: parse_positive(bot_max_tokens).unwrap_or(defaults.max_tokens),
            max_seconds: parse_positive(bot_max_seconds).unwrap_or(defaults.max_seconds),
        }
    }

    /// Applies group policies, keeping the most restrictive non-zero limit.
    /// Group limits can only tighten the current caps, never raise them.
    #[must_use]
    pub fn restrict_to(self, max_tokens: Option<u64>, max_seconds: Option<u64>) -> Self {
        let tighten = |current: u64, candidate: Option<u64>| match candidate.filter(|value| *value > 0) {
            Some(value) if current == 0 || value < current => value,
            _ => current,
        };
        Self {
            max_tokens: tighten(self.max_tokens, max_tokens),
            max_seconds: tighten(self.max_seconds, max_seconds),
        }
    }

    #[must_use]
    pub fn is_unlimited(&self) -> bool {
        self.max_tokens == 0 && self.max_seconds == 0
    }
}

fn parse_positive(value: Option<&str>) -> Option<u64> {
    value
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|parsed| *parsed > 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapKind {
    Tokens,
    Seconds,
}

/// What stopped the run and how far it got.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapBreach {
    pub kind: CapKind,
    pub limit: u64,
    pub observed: u64,
}

/// Mutable per-run budget. Cheap: two counters and a start instant.
#[derive(Debug)]
pub struct RunBudget {
    caps: RunCaps,
    start: Instant,
    chars: u64,
}

impl RunBudget {
    #[must_use]
    pub fn start(caps: RunCaps) -> Self {
        Self {
            caps,
            start: Instant::now(),
            chars: 0,
        }
    }

    /// Feeds the number of content characters produced so far.
    pub fn observe_chars(&mut self, total_chars: usize) {
        self.chars = u64::try_from(total_chars).unwrap_or(u64::MAX);
    }

    /// Approximate tokens: four characters per token, rounded up.
    #[must_use]
    pub fn observed_tokens(&self) -> u64 {
        self.chars.div_ceil(4)
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Returns the first breached limit, tokens before seconds.
    #[must_use]
    pub fn check(&self) -> Option<CapBreach> {
        let tokens = self.observed_tokens();
        if self.caps.max_tokens > 0 && tokens >= self.caps.max_tokens {
            return Some(CapBreach {
                kind: CapKind::Tokens,
                limit: self.caps.max_tokens,
                observed: tokens,
            });
        }
        let seconds = self.elapsed().as_secs();
        if self.caps.max_seconds > 0 && seconds >= self.caps.max_seconds {
            return Some(CapBreach {
                kind: CapKind::Seconds,
                limit: self.caps.max_seconds,
                observed: seconds,
            });
        }
        None
    }

    /// User-facing explanation persisted with the stopped run.
    #[must_use]
    pub fn describe(breach: &CapBreach) -> String {
        match breach.kind {
            CapKind::Tokens => format!(
                "token cap reached ({} tokens; cap {})",
                breach.observed, breach.limit
            ),
            CapKind::Seconds => format!(
                "time cap reached ({}s; cap {}s)",
                breach.observed, breach.limit
            ),
        }
    }
}

fn env_u64(key: &str) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlimited_budget_never_breaches() {
        let mut budget = RunBudget::start(RunCaps::default());
        budget.observe_chars(10_000_000);
        assert!(budget.check().is_none());
        assert!(RunCaps::default().is_unlimited());
    }

    #[test]
    fn token_cap_stops_at_the_limit() {
        let caps = RunCaps {
            max_tokens: 10,
            max_seconds: 0,
        };
        let mut budget = RunBudget::start(caps);
        budget.observe_chars(36); // 9 tokens
        assert!(budget.check().is_none());
        budget.observe_chars(40); // 10 tokens
        let breach = budget.check().expect("cap must trigger");
        assert_eq!(breach.kind, CapKind::Tokens);
        assert_eq!(breach.observed, 10);
        assert!(RunBudget::describe(&breach).contains("token cap reached"));
    }

    #[test]
    fn token_estimate_rounds_up() {
        let mut budget = RunBudget::start(RunCaps::default());
        budget.observe_chars(1);
        assert_eq!(budget.observed_tokens(), 1);
        budget.observe_chars(5);
        assert_eq!(budget.observed_tokens(), 2);
    }

    #[test]
    fn time_cap_uses_elapsed_wall_clock() {
        let caps = RunCaps {
            max_tokens: 0,
            max_seconds: 2,
        };
        let budget = RunBudget {
            caps,
            start: Instant::now() - Duration::from_secs(3),
            chars: 0,
        };
        let breach = budget.check().expect("time cap must trigger");
        assert_eq!(breach.kind, CapKind::Seconds);
        assert!(RunBudget::describe(&breach).contains("time cap reached"));
    }

    #[test]
    fn tokens_are_reported_before_seconds() {
        let caps = RunCaps {
            max_tokens: 1,
            max_seconds: 1,
        };
        let budget = RunBudget {
            caps,
            start: Instant::now() - Duration::from_secs(5),
            chars: 4,
        };
        assert_eq!(budget.check().map(|b| b.kind), Some(CapKind::Tokens));
    }

    #[test]
    fn bot_settings_override_environment_defaults() {
        let defaults = RunCaps {
            max_tokens: 1000,
            max_seconds: 60,
        };
        let caps = RunCaps::merge(Some("250"), Some("30"), defaults);
        assert_eq!(caps.max_tokens, 250);
        assert_eq!(caps.max_seconds, 30);
    }

    #[test]
    fn missing_or_invalid_bot_settings_keep_defaults() {
        let defaults = RunCaps {
            max_tokens: 1000,
            max_seconds: 60,
        };
        assert_eq!(RunCaps::merge(None, None, defaults), defaults);
        assert_eq!(RunCaps::merge(Some(""), Some("not-a-number"), defaults), defaults);
        assert_eq!(RunCaps::merge(Some("0"), Some("-5"), defaults), defaults);
    }

    #[test]
    fn group_policies_only_tighten_caps() {
        let defaults = RunCaps {
            max_tokens: 1000,
            max_seconds: 60,
        };
        let tightened = defaults.restrict_to(Some(200), Some(600));
        assert_eq!(tightened.max_tokens, 200);
        // A looser seconds limit cannot raise the existing 60s cap.
        assert_eq!(tightened.max_seconds, 60);
    }

    #[test]
    fn group_policy_sets_an_unlimited_default() {
        let caps = RunCaps::default().restrict_to(Some(500), Some(30));
        assert_eq!(caps.max_tokens, 500);
        assert_eq!(caps.max_seconds, 30);
        assert!(!caps.is_unlimited());
    }
}
