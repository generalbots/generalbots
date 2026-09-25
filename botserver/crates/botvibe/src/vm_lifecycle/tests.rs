//! `vm_lifecycle::tests` — split per #1443.

use super::*;

    use super::*;

    #[test]
    fn protected_container_names_are_rejected() {
        // #1397 — platform service containers must never be claimed by projects
        for name in ["tables", "system", "bot", "proxy", "TABLES", "vault"] {
            assert!(VmLifecycle::is_protected_container_name(name), "{name} should be protected");
        }
        for name in ["my-app", "contato-calc", "expense-tracker"] {
            assert!(!VmLifecycle::is_protected_container_name(name), "{name} should be allowed");
        }
    }

    #[test]
    fn container_names_are_env_scoped() {
        assert_eq!(
            VmLifecycle::container_name("My App", "development", false),
            "my-app-development"
        );
        assert_eq!(
            VmLifecycle::container_name("My App", "production", false),
            "my-app-prod"
        );
        assert_eq!(
            VmLifecycle::container_name("Web", "development", true),
            "web-development-runner"
        );
    }

    #[test]
    fn validate_accepts_known_envs_and_tiers() {
        let ok_req = CreateVmRequest {
            env: "production".into(),
            tier: "medium".into(),
            runner_enabled: false,
        };
        assert!(VmLifecycle::validate(&ok_req).is_ok());
        let bad = CreateVmRequest {
            env: "moon".into(),
            tier: "small".into(),
            runner_enabled: false,
        };
        assert!(VmLifecycle::validate(&bad).is_err());
        let bad_tier = CreateVmRequest {
            env: "dev".into(),
            tier: "huge".into(),
            runner_enabled: false,
        };
        assert!(VmLifecycle::validate(&bad_tier).is_err());
    }

    #[test]
    fn alm_mapping_org_is_branch_short() {
        let id = Uuid::new_v4();
        let org = VmLifecycle::alm_org(id);
        assert_eq!(org.len(), 8);
        assert_eq!(VmLifecycle::alm_repo("My Web App"), "my-web-app");
    }

    #[test]
    fn skip_pattern_reports_unavailable_when_forced() {
        std::env::set_var("VIBE_INCUS_FORCE_UNAVAILABLE", "1");
        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
            "postgres://127.0.0.1:1/nonexistent",
        );
        let pool = diesel::r2d2::Pool::builder()
            .max_size(1)
            .min_idle(Some(0))
            .build(manager)
            .expect("pool builder");
        let lifecycle = VmLifecycle::new(pool);
        assert!(!lifecycle.linux_available());
        let err = lifecycle.linux_running("x").unwrap_err();
        assert!(err.contains("vm-skip"), "expected skip error, got {err}");
        std::env::remove_var("VIBE_INCUS_FORCE_UNAVAILABLE");
    }

