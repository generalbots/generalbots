//! `api::tests` — split per #1443.

use super::*;
use super::run::derive_project_name;



    #[test]
    fn truncate_chars_never_panics_on_multibyte_boundary() {
        // 79 ASCII chars + a 2-byte 'é' at the boundary would panic under
        // byte slicing; the char-safe cut must not.
        let s = format!("{}é", "a".repeat(79));
        let out = truncate_chars(&s, 80);
        assert!(s.starts_with(out));
        assert!(out.len() <= 80);
        assert_eq!(truncate_chars("short", 80), "short");
        assert_eq!(truncate_chars("", 80), "");
    }

    #[test]
    fn truncate_chars_handles_portuguese_and_emoji() {
        // 'coração' has multi-byte 'ç'/'ã'; '🚀' is a 4-byte emoji. Both must
        // survive a cut that lands inside them without panicking.
        let pt = "eu quero agendar um batizado na catedral da sé, coração".repeat(4);
        assert!(truncate_chars(&pt, 80).len() <= 80);
        let emoji = format!("{}🚀", "x".repeat(79));
        assert!(truncate_chars(&emoji, 80).len() <= 80);
    }

    #[test]
    fn derive_project_name_strips_stopwords_and_slugs() {
        assert_eq!(
            derive_project_name("Create a calculator web app with + - * / buttons"),
            "calculator-web-app"
        );
        assert_eq!(
            derive_project_name("Build a new landing page"),
            "landing-page"
        );
        assert_eq!(derive_project_name("refactor the auth module"), "refactor-auth-module");
        assert_eq!(derive_project_name(""), "app");
        assert_eq!(derive_project_name("   "), "app");
        // #1272 — deictic phrases never become project names.
        assert_eq!(
            derive_project_name("Deploy the selected project to production"),
            "deploy-to-production"
        );
        assert_eq!(
            derive_project_name("Update this project settings"),
            "update-settings"
        );
    }

    /// #2541 — boot rebuild restores persisted runs and keeps the fresher
    /// in-memory copy when a run id is present in both sources.
    #[test]
    fn rebuild_run_map_restores_persisted_and_prefers_live() {
        use crate::types::VibeRunConfig;

        let persisted = VibeRun::new(
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
            "persisted".to_string(),
            VibeRunConfig::default(),
        );
        let other = VibeRun::new(
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
            "other".to_string(),
            VibeRunConfig::default(),
        );
        let shared_id = persisted.run_id;
        let mut live = VibeRun::new(
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
            "live".to_string(),
            VibeRunConfig::default(),
        );
        live.run_id = shared_id;
        live.transition(VibeRunState::Running);

        let rebuilt = rebuild_run_map(
            vec![persisted, other.clone()],
            HashMap::from([(shared_id, live)]),
        );

        assert_eq!(rebuilt.len(), 2);
        assert_eq!(rebuilt.get(&shared_id).map(|r| r.intent.as_str()), Some("live"));
        assert!(rebuilt.get(&other.run_id).is_some());
    }

    /// #2541 — with no live runs the rebuilt map is exactly the persisted
    /// snapshot (the boot case: the in-memory map starts empty on restart).
    #[test]
    fn rebuild_run_map_from_empty_live_is_persisted_snapshot() {
        use crate::types::VibeRunConfig;

        let persisted = VibeRun::new(
            Uuid::nil(),
            Uuid::nil(),
            Uuid::nil(),
            "persisted".to_string(),
            VibeRunConfig::default(),
        );
        let id = persisted.run_id;
        let rebuilt = rebuild_run_map(vec![persisted], HashMap::new());
        assert_eq!(rebuilt.len(), 1);
        assert_eq!(rebuilt.get(&id).map(|r| r.intent.as_str()), Some("persisted"));
    }

