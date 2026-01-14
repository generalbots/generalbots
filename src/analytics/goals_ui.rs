use axum::{
    extract::{Query, State},
    response::Html,
    routing::get,
    Router,
};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{okr_checkins, okr_objectives};
use crate::shared::state::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct ObjectivesQuery {
    pub status: Option<String>,
    pub period: Option<String>,
    pub owner_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn objectives_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ObjectivesQuery>,
) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        let mut db_query = okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(status) = query.status {
            db_query = db_query.filter(okr_objectives::status.eq(status));
        }
        if let Some(period) = query.period {
            db_query = db_query.filter(okr_objectives::period.eq(period));
        }
        if let Some(owner_id) = query.owner_id {
            db_query = db_query.filter(okr_objectives::owner_id.eq(owner_id));
        }

        db_query = db_query.order(okr_objectives::created_at.desc());

        if let Some(limit) = query.limit {
            db_query = db_query.limit(limit);
        } else {
            db_query = db_query.limit(50);
        }

        db_query
            .select((
                okr_objectives::id,
                okr_objectives::title,
                okr_objectives::description,
                okr_objectives::period,
                okr_objectives::status,
                okr_objectives::progress,
                okr_objectives::visibility,
                okr_objectives::created_at,
            ))
            .load::<(Uuid, String, Option<String>, String, String, bigdecimal::BigDecimal, String, DateTime<Utc>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(objectives) if !objectives.is_empty() => {
            let items: String = objectives
                .iter()
                .map(|(id, title, _desc, period, status, progress, visibility, _created)| {
                    let progress_val = progress.to_f32().unwrap_or(0.0);
                    let progress_pct = (progress_val * 100.0) as i32;
                    let status_class = match status.as_str() {
                        "active" => "status-active",
                        "on_track" => "status-on-track",
                        "at_risk" => "status-at-risk",
                        "behind" => "status-behind",
                        "completed" => "status-completed",
                        _ => "status-draft",
                    };
                    let progress_class = if progress_val >= 0.7 {
                        "progress-good"
                    } else if progress_val >= 0.4 {
                        "progress-medium"
                    } else {
                        "progress-low"
                    };

                    format!(
                        r##"<div class="objective-card" data-id="{id}" hx-get="/api/ui/goals/objectives/{id}" hx-target="#objective-detail" hx-swap="innerHTML">
                            <div class="objective-header">
                                <h4 class="objective-title">{title}</h4>
                                <span class="objective-status {status_class}"><span class="status-dot"></span>{status}</span>
                            </div>
                            <div class="objective-meta">
                                <span class="objective-period">{period}</span>
                                <span class="objective-visibility">{visibility}</span>
                            </div>
                            <div class="objective-progress">
                                <div class="progress-bar {progress_class}">
                                    <div class="progress-fill" style="width: {progress_pct}%;"></div>
                                </div>
                                <span class="progress-text">{progress_pct}%</span>
                            </div>
                        </div>"##
                    )
                })
                .collect();

            Html(format!(r##"<div class="objectives-list">{items}</div>"##))
        }
        _ => Html(
            r##"<div class="empty-state">
                <p>No objectives found</p>
                <button class="btn btn-primary" hx-get="/api/ui/goals/new-objective" hx-target="#modal-content">
                    Create Objective
                </button>
            </div>"##.to_string(),
        ),
    }
}

pub async fn objectives_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

pub async fn active_objectives_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("active"))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

pub async fn at_risk_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("at_risk"))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

pub async fn average_progress(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        let objectives = okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.ne("draft"))
            .filter(okr_objectives::status.ne("cancelled"))
            .select(okr_objectives::progress)
            .load::<bigdecimal::BigDecimal>(&mut conn)
            .ok()?;

        if objectives.is_empty() {
            return Some(0.0f32);
        }

        let sum: f32 = objectives.iter().map(|p| p.to_f32().unwrap_or(0.0)).sum();
        Some(sum / objectives.len() as f32)
    })
    .await
    .ok()
    .flatten();

    let avg = result.unwrap_or(0.0);
    let pct = (avg * 100.0) as i32;
    Html(format!("{pct}%"))
}

pub async fn dashboard_stats(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        let total: i64 = okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let active: i64 = okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("active"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let at_risk: i64 = okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("at_risk"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let completed: i64 = okr_objectives::table
            .filter(okr_objectives::bot_id.eq(bot_id))
            .filter(okr_objectives::status.eq("completed"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        Some((total, active, at_risk, completed))
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((total, active, at_risk, completed)) => Html(format!(
            r##"<div class="dashboard-stats">
                <div class="stat-card">
                    <span class="stat-value">{total}</span>
                    <span class="stat-label">Total Objectives</span>
                </div>
                <div class="stat-card stat-success">
                    <span class="stat-value">{active}</span>
                    <span class="stat-label">Active</span>
                </div>
                <div class="stat-card stat-warning">
                    <span class="stat-value">{at_risk}</span>
                    <span class="stat-label">At Risk</span>
                </div>
                <div class="stat-card stat-info">
                    <span class="stat-value">{completed}</span>
                    <span class="stat-label">Completed</span>
                </div>
            </div>"##
        )),
        None => Html(r##"<div class="dashboard-stats"><div class="stat-card"><span class="stat-value">-</span></div></div>"##.to_string()),
    }
}

pub async fn new_objective_form() -> Html<String> {
    Html(r##"<div class="modal-header">
        <h3>New Objective</h3>
        <button class="btn-close" onclick="closeModal()">&times;</button>
    </div>
    <form class="objective-form" hx-post="/api/goals/objectives" hx-swap="none">
        <div class="form-group">
            <label>Title</label>
            <input type="text" name="title" placeholder="What do you want to achieve?" required />
        </div>
        <div class="form-group">
            <label>Description</label>
            <textarea name="description" rows="3" placeholder="Describe the objective in detail"></textarea>
        </div>
        <div class="form-group">
            <label>Period</label>
            <select name="period" required>
                <option value="Q1">Q1</option>
                <option value="Q2">Q2</option>
                <option value="Q3">Q3</option>
                <option value="Q4">Q4</option>
                <option value="H1">H1 (Half Year)</option>
                <option value="H2">H2 (Half Year)</option>
                <option value="annual">Annual</option>
            </select>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label>Start Date</label>
                <input type="date" name="period_start" />
            </div>
            <div class="form-group">
                <label>End Date</label>
                <input type="date" name="period_end" />
            </div>
        </div>
        <div class="form-group">
            <label>Visibility</label>
            <select name="visibility">
                <option value="team">Team</option>
                <option value="organization">Organization</option>
                <option value="private">Private</option>
            </select>
        </div>
        <div class="form-actions">
            <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
            <button type="submit" class="btn btn-primary">Create Objective</button>
        </div>
    </form>"##.to_string())
}

pub async fn recent_checkins(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = Some(get_default_bot(&mut conn))?;

        okr_checkins::table
            .filter(okr_checkins::bot_id.eq(bot_id))
            .order(okr_checkins::created_at.desc())
            .limit(10)
            .select((
                okr_checkins::id,
                okr_checkins::new_value,
                okr_checkins::note,
                okr_checkins::confidence,
                okr_checkins::created_at,
            ))
            .load::<(Uuid, bigdecimal::BigDecimal, Option<String>, Option<String>, DateTime<Utc>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(checkins) if !checkins.is_empty() => {
            let items: String = checkins
                .iter()
                .map(|(id, value, note, confidence, created)| {
                    let val = value.to_f64().unwrap_or(0.0);
                    let note_text = note.clone().unwrap_or_else(|| "No note".to_string());
                    let conf = confidence.clone().unwrap_or_else(|| "medium".to_string());
                    let conf_class = match conf.as_str() {
                        "high" => "confidence-high",
                        "low" => "confidence-low",
                        _ => "confidence-medium",
                    };
                    let time_str = created.format("%b %d, %H:%M").to_string();

                    format!(
                        r##"<div class="checkin-item" data-id="{id}">
                            <div class="checkin-header">
                                <span class="checkin-value">{val:.2}</span>
                                <span class="checkin-confidence {conf_class}">{conf}</span>
                            </div>
                            <p class="checkin-note">{note_text}</p>
                            <span class="checkin-time">{time_str}</span>
                        </div>"##
                    )
                })
                .collect();

            Html(format!(r##"<div class="checkins-list">{items}</div>"##))
        }
        _ => Html(r##"<div class="empty-state"><p>No recent check-ins</p></div>"##.to_string()),
    }
}

pub fn configure_goals_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/goals/objectives", get(objectives_list))
        .route("/api/ui/goals/objectives/count", get(objectives_count))
        .route("/api/ui/goals/objectives/active", get(active_objectives_count))
        .route("/api/ui/goals/objectives/at-risk", get(at_risk_count))
        .route("/api/ui/goals/dashboard", get(dashboard_stats))
        .route("/api/ui/goals/progress", get(average_progress))
        .route("/api/ui/goals/checkins/recent", get(recent_checkins))
        .route("/api/ui/goals/new-objective", get(new_objective_form))
}
