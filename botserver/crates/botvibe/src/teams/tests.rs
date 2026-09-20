//! `teams::tests` — split per #1443.

use super::*;

    use super::*;

    fn sample_team(name: &str) -> VibeTeam {
        VibeTeam {
            team_id: Uuid::new_v4(),
            name: name.to_string(),
            objective: "ship".to_string(),
            members: vec![
                TeamMember { name: "alice".into(), task: "t1".into(), run_id: None, state: "pending".into(), error: None },
                TeamMember { name: "bob".into(), task: "t2".into(), run_id: None, state: "pending".into(), error: None },
            ],
            shared_tasks: vec!["t1".into(), "t2".into()],
            status: "running".into(),
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    #[tokio::test]
    async fn insert_get_list_order() {
        let store = TeamStore::new();
        let older = sample_team("older");
        store.insert(older.clone()).await;
        let newer = sample_team("newer");
        store.insert(newer.clone()).await;
        assert_eq!(store.get(newer.team_id).await.unwrap().name, "newer");
        let list = store.list().await;
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].team_id, newer.team_id);
        assert!(store.get(Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn update_member_marks_team_failed_when_any_member_fails() {
        let store = TeamStore::new();
        let team = sample_team("t");
        store.insert(team.clone()).await;
        store.update_member(team.team_id, 0, TeamMember { name: "alice".into(), task: "t1".into(), run_id: Some(Uuid::new_v4()), state: "completed".into(), error: None }).await;
        let mid = store.get(team.team_id).await.unwrap();
        assert_eq!(mid.status, "running");
        store.update_member(team.team_id, 1, TeamMember { name: "bob".into(), task: "t2".into(), run_id: None, state: "failed".into(), error: Some("boom".into()) }).await;
        let done = store.get(team.team_id).await.unwrap();
        assert_eq!(done.status, "failed");
        assert!(done.completed_at.is_some());
    }

    #[tokio::test]
    async fn update_member_marks_team_completed_when_all_members_complete() {
        let store = TeamStore::new();
        let team = sample_team("t");
        store.insert(team.clone()).await;
        store.update_member(team.team_id, 0, TeamMember { name: "alice".into(), task: "t1".into(), run_id: Some(Uuid::new_v4()), state: "completed".into(), error: None }).await;
        store.update_member(team.team_id, 1, TeamMember { name: "bob".into(), task: "t2".into(), run_id: Some(Uuid::new_v4()), state: "completed".into(), error: None }).await;
        let done = store.get(team.team_id).await.unwrap();
        assert_eq!(done.status, "completed");
        assert!(done.completed_at.is_some());
    }

    #[test]
    fn team_status_aggregates_member_states() {
        fn member(state: &str) -> TeamMember {
            TeamMember { name: "x".into(), task: "t".into(), run_id: None, state: state.into(), error: None }
        }
        assert_eq!(team_status(&[member("completed"), member("completed")]), "completed");
        assert_eq!(team_status(&[member("completed"), member("failed")]), "failed");
        assert_eq!(team_status(&[member("running"), member("pending")]), "running");
        assert_eq!(team_status(&[]), "completed", "empty team is trivially done");
    }

    #[tokio::test]
    async fn update_member_ignores_unknown_team() {
        let store = TeamStore::new();
        store.update_member(Uuid::new_v4(), 0, TeamMember { name: "x".into(), task: "t".into(), run_id: None, state: "completed".into(), error: None }).await;
        assert!(store.list().await.is_empty());
    }

