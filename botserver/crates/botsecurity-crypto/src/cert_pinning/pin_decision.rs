//! Pure pin-decision predicate shared by the TLS verifier and the manager.
//!
//! Keeping the decision a side-effect-free function makes the security-critical
//! "accept or reject this certificate" choice testable without a live TLS
//! handshake (issue #2626).

use super::*;

/// The outcome of evaluating a certificate against a host's pin policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinDecision {
    Accept,
    Reject,
}

/// Decide whether a TLS handshake may proceed.
///
/// # Arguments
/// * `enabled` - whether pinning is active at all
/// * `report_only` - log violations but never block (monitoring rollout)
/// * `require_pins` - strict mode: a host with no pins is rejected
/// * `has_pins_for_host` - the config has at least one pin for this hostname
/// * `pin_matched` - the presented certificate matched one of those pins
///
/// Fail-closed rules: in strict mode a host without pins is rejected; when pins
/// exist for a host but none match, the certificate is rejected. `report_only`
/// downgrades every rejection to an acceptance (the caller still logs it), and
/// `enabled = false` defers entirely to the normal system trust store.
pub fn pin_decision(
    enabled: bool,
    report_only: bool,
    require_pins: bool,
    has_pins_for_host: bool,
    pin_matched: bool,
) -> PinDecision {
    if !enabled {
        return PinDecision::Accept;
    }
    if pin_matched {
        return PinDecision::Accept;
    }

    let must_reject = require_pins || has_pins_for_host;
    if must_reject && !report_only {
        PinDecision::Reject
    } else {
        PinDecision::Accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_defers_to_system_trust() {
        // Even an obvious mismatch is accepted when pinning is off.
        assert_eq!(
            pin_decision(false, false, true, true, false),
            PinDecision::Accept
        );
    }

    #[test]
    fn matching_pin_always_accepts() {
        assert_eq!(
            pin_decision(true, false, true, true, true),
            PinDecision::Accept
        );
        assert_eq!(
            pin_decision(true, false, false, true, true),
            PinDecision::Accept
        );
    }

    #[test]
    fn no_pins_and_not_required_accepts() {
        assert_eq!(
            pin_decision(true, false, false, false, false),
            PinDecision::Accept
        );
    }

    #[test]
    fn strict_mode_without_pins_fails_closed() {
        assert_eq!(
            pin_decision(true, false, true, false, false),
            PinDecision::Reject
        );
    }

    #[test]
    fn configured_pins_with_no_match_fails_closed() {
        assert_eq!(
            pin_decision(true, false, false, true, false),
            PinDecision::Reject
        );
    }

    #[test]
    fn report_only_downgrades_every_rejection() {
        assert_eq!(
            pin_decision(true, true, true, false, false),
            PinDecision::Accept
        );
        assert_eq!(
            pin_decision(true, true, false, true, false),
            PinDecision::Accept
        );
    }
}
