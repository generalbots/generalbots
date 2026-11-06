use std::collections::HashMap;
use std::time::Instant;

use uuid::Uuid;

use crate::models::{ConsentScope, Decision};

use super::resolve::resolve_in_memory;
use super::service::{sweep_maps, ttl};
use super::{PendingMap, PendingRequest, ResolvedOutcome};

    fn req(id: &str) -> PendingRequest {
        PendingRequest {
            request_id: id.to_string(),
            user_id: Uuid::nil(),
            app_id: "crm".to_string(),
            action_class: "create".to_string(),
            detail: serde_json::json!({ "name": "cmd" }),
            thread_id: None,
            run_id: None,
            risk_tier: "medium".to_string(),
        }
    }

    fn maps() -> (PendingMap, PendingMap) {
        (HashMap::new(), HashMap::new())
    }

    #[test]
    fn allow_once_marks_consumed_exactly_once() {
        let (mut p, mut a) = maps();
        p.insert("r1".into(), (req("r1"), Instant::now()));
        let out = resolve_in_memory(&mut p, &mut a, "r1", Decision::AllowOnce, Instant::now())
            .expect("transition ok");
        assert!(matches!(out, ResolvedOutcome::ConsumeOnce(_)));
        assert!(!p.contains_key("r1"));
        assert_eq!(
            take_from(&mut a, "r1").map(|r| r.request_id),
            Some("r1".to_string())
        );
        assert!(take_from(&mut a, "r1").is_none());
    }

    #[test]
    fn always_does_not_populate_approved_once() {
        let (mut p, mut a) = maps();
        p.insert("r2".into(), (req("r2"), Instant::now()));
        let out = resolve_in_memory(&mut p, &mut a, "r2", Decision::Always, Instant::now())
            .expect("transition ok");
        assert_eq!(out, ResolvedOutcome::PersistGrant(req("r2"), ConsentScope::Always));
        assert!(a.is_empty());
        assert!(p.is_empty());
    }

    #[test]
    fn deny_records_without_approval() {
        let (mut p, mut a) = maps();
        p.insert("r3".into(), (req("r3"), Instant::now()));
        let out = resolve_in_memory(&mut p, &mut a, "r3", Decision::Deny, Instant::now())
            .expect("transition ok");
        assert!(matches!(out, ResolvedOutcome::RecordDenial(_)));
        assert!(a.is_empty());
        assert!(p.is_empty());
    }

    #[test]
    fn unknown_and_expired_requests_are_rejected() {
        let (mut p, mut a) = maps();
        assert!(resolve_in_memory(&mut p, &mut a, "ghost", Decision::AllowOnce, Instant::now()).is_err());

        let stale_born = Instant::now().checked_sub(ttl()).unwrap_or_else(Instant::now);
        p.insert("old".into(), (req("old"), stale_born));
        assert!(resolve_in_memory(&mut p, &mut a, "old", Decision::AllowOnce, Instant::now()).is_err());
    }

    #[test]
    fn sweeper_drops_expired_entries_only() {
        let (mut p, mut a) = maps();
        let fresh = Instant::now();
        let stale = fresh.checked_sub(ttl()).unwrap_or(fresh);
        p.insert("fresh".into(), (req("fresh"), fresh));
        p.insert("stale".into(), (req("stale"), stale));
        a.insert("astale".into(), (req("astale"), stale));
        sweep_maps(&mut p, &mut a, fresh);
        assert!(p.contains_key("fresh"));
        assert!(!p.contains_key("stale"));
        assert!(!a.contains_key("astale"));
    }

    fn take_from(
        a: &mut PendingMap,
        id: &str,
    ) -> Option<PendingRequest> {
        a.remove(id).map(|(req, _)| req)
    }
