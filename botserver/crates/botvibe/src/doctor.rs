use crate::sessions::SessionStore;
use crate::skills::{SkillStore, VibeSkill};
use axum::{Extension, Json, Router};
use serde::Serialize;
use std::sync::Arc;

const STALE_SESSION_SECS: i64 = 24 * 3600;

#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub overall: String,
    pub checks: Vec<DoctorCheck>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub fn check_skills_unused(skills: &[VibeSkill]) -> Vec<String> {
    skills
        .iter()
        .filter(|s| s.triggers.is_empty() && s.enabled)
        .map(|s| s.name.clone())
        .collect()
}

pub fn check_skills_overlap(skills: &[VibeSkill]) -> Vec<String> {
    let mut overlaps = Vec::new();
    for i in 0..skills.len() {
        for j in (i + 1)..skills.len() {
            let common: Vec<&String> = skills[i]
                .triggers
                .iter()
                .filter(|t| skills[j].triggers.iter().any(|o| o.to_lowercase() == t.to_lowercase()))
                .collect();
            for trigger in common {
                overlaps.push(format!(
                    "{} and {} both trigger on '{}'",
                    skills[i].name, skills[j].name, trigger
                ));
            }
        }
    }
    overlaps
}

pub fn check_sessions_stale(sessions: &[crate::sessions::VibeSession]) -> usize {
    let cutoff = chrono::Utc::now() - chrono::Duration::seconds(STALE_SESSION_SECS);
    sessions.iter().filter(|s| s.updated_at < cutoff).count()
}

pub async fn run_doctor(skills: &SkillStore, sessions: &SessionStore) -> DoctorReport {
    let skill_list = skills.list().await;
    let session_list = sessions.list().await;
    let mut checks = Vec::new();

    if skill_list.is_empty() {
        checks.push(DoctorCheck {
            name: "skills-count".into(),
            status: "warn".into(),
            detail: "no skills registered; run seed_bootstrap or create one".into(),
        });
    } else {
        checks.push(DoctorCheck {
            name: "skills-count".into(),
            status: "ok".into(),
            detail: format!("{} skills registered", skill_list.len()),
        });
    }

    let unused = check_skills_unused(&skill_list);
    if unused.is_empty() {
        checks.push(DoctorCheck {
            name: "skills-unused".into(),
            status: "ok".into(),
            detail: "all enabled skills have triggers".into(),
        });
    } else {
        checks.push(DoctorCheck {
            name: "skills-unused".into(),
            status: "warn".into(),
            detail: format!("skills without triggers: {}", unused.join(", ")),
        });
    }

    let overlap = check_skills_overlap(&skill_list);
    if overlap.is_empty() {
        checks.push(DoctorCheck {
            name: "skills-overlap".into(),
            status: "ok".into(),
            detail: "no overlapping skill triggers".into(),
        });
    } else {
        checks.push(DoctorCheck {
            name: "skills-overlap".into(),
            status: "warn".into(),
            detail: overlap.join("; "),
        });
    }

    let stale = check_sessions_stale(&session_list);
    if stale == 0 {
        checks.push(DoctorCheck {
            name: "sessions-stale".into(),
            status: "ok".into(),
            detail: "no stale sessions".into(),
        });
    } else {
        checks.push(DoctorCheck {
            name: "sessions-stale".into(),
            status: "warn".into(),
            detail: format!("{stale} sessions idle for over 24h"),
        });
    }

    let workspace = std::env::var("VIBE_WORKSPACE_ROOT").unwrap_or_default();
    if workspace.is_empty() {
        checks.push(DoctorCheck {
            name: "workspace-root".into(),
            status: "warn".into(),
            detail: "VIBE_WORKSPACE_ROOT not set; tools fall back to a temp dir".into(),
        });
    } else {
        let path = std::path::Path::new(&workspace);
        let writable = path.is_dir() && std::fs::write(path.join(".doctor-probe"), b"ok").is_ok();
        if writable {
            checks.push(DoctorCheck {
                name: "workspace-root".into(),
                status: "ok".into(),
                detail: format!("{workspace} is writable"),
            });
        } else {
            checks.push(DoctorCheck {
                name: "workspace-root".into(),
                status: "warn".into(),
                detail: format!("{workspace} is not writable"),
            });
        }
    }

    let git_ok = crate::harness::cmd::run(
        "git",
        &["--version".into()],
        std::path::Path::new("."),
        5,
    )
    .is_ok();
    checks.push(DoctorCheck {
        name: "env-git".into(),
        status: if git_ok { "ok".into() } else { "warn".into() },
        detail: if git_ok { "git available".into() } else { "git unavailable".into() },
    });

    let incus_ok = crate::harness::cmd::run(
        "incus",
        &["version".into()],
        std::path::Path::new("."),
        5,
    )
    .is_ok();
    checks.push(DoctorCheck {
        name: "env-incus".into(),
        status: if incus_ok { "ok".into() } else { "warn".into() },
        detail: if incus_ok {
            "incus available; VM management enabled".into()
        } else {
            "incus unavailable; VM operations skip gracefully".into()
        },
    });

    let overall = if checks.iter().all(|c| c.status == "ok") {
        "ok".to_string()
    } else if checks.iter().any(|c| c.status == "error") {
        "error".to_string()
    } else {
        "warn".to_string()
    };

    DoctorReport {
        overall,
        checks,
        timestamp: chrono::Utc::now(),
    }
}

pub fn doctor_router(skills: Arc<SkillStore>, sessions: Arc<SessionStore>) -> Router {
    Router::new()
        .route("/api/vibe/doctor", axum::routing::get(handle_doctor))
        .layer(Extension(skills))
        .layer(Extension(sessions))
}

async fn handle_doctor(
    Extension(skills): Extension<Arc<SkillStore>>,
    Extension(sessions): Extension<Arc<SessionStore>>,
) -> Json<DoctorReport> {
    Json(run_doctor(&skills, &sessions).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(name: &str, triggers: &[&str]) -> VibeSkill {
        VibeSkill {
            skill_id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            content: String::new(),
            triggers: triggers.iter().map(|t| t.to_string()).collect(),
            enabled: true,
        }
    }

    #[test]
    fn unused_skills_are_reported() {
        let skills = vec![skill("no-trigger", &[]), skill("ok", &["deploy"])];
        let unused = check_skills_unused(&skills);
        assert_eq!(unused, vec!["no-trigger".to_string()]);
    }

    #[test]
    fn overlapping_triggers_are_reported() {
        let skills = vec![skill("a", &["deploy"]), skill("b", &["Deploy", "lint"]), skill("c", &["lint"])];
        let overlaps = check_skills_overlap(&skills);
        assert_eq!(overlaps.len(), 2);
        assert!(overlaps[0].contains("a") && overlaps[0].contains("b"));
        assert!(overlaps[1].contains("b") && overlaps[1].contains("c"));
    }

    #[test]
    fn no_overlap_when_disjoint() {
        let skills = vec![skill("a", &["x"]), skill("b", &["y"])];
        assert!(check_skills_overlap(&skills).is_empty());
    }

    #[test]
    fn stale_sessions_counted() {
        let old = chrono::Utc::now() - chrono::Duration::hours(48);
        let fresh = chrono::Utc::now();
        let sessions = vec![
            crate::sessions::VibeSession {
                session_id: Uuid::new_v4(),
                parent_session_id: None,
                bot_id: Uuid::nil(),
                user_id: Uuid::nil(),
                intent: "old".into(),
                use_case: crate::types::VibeUseCase::SoftwareDevelopment,
                budget_cents: 0,
                run: None,
                created_at: old,
                updated_at: old,
            },
            crate::sessions::VibeSession {
                session_id: Uuid::new_v4(),
                parent_session_id: None,
                bot_id: Uuid::nil(),
                user_id: Uuid::nil(),
                intent: "fresh".into(),
                use_case: crate::types::VibeUseCase::SoftwareDevelopment,
                budget_cents: 0,
                run: None,
                created_at: fresh,
                updated_at: fresh,
            },
        ];
        assert_eq!(check_sessions_stale(&sessions), 1);
    }

    #[tokio::test]
    async fn run_doctor_reports_system_checks() {
        let skills = SkillStore::new();
        skills.seed_bootstrap().await;
        let sessions = SessionStore::new();
        let report = run_doctor(&skills, &sessions).await;
        assert_eq!(report.overall, "ok");
        let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
        for expected in ["skills-count", "skills-unused", "skills-overlap", "sessions-stale", "workspace-root", "env-git", "env-incus"] {
            assert!(names.contains(&expected), "missing check {expected}");
        }
    }
}
