use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{people, people_departments, people_teams, people_time_off};
use crate::shared::state::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct PeopleQuery {
    pub department: Option<String>,
    pub team_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn configure_people_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/people", get(handle_people_list))
        .route("/api/ui/people/count", get(handle_people_count))
        .route("/api/ui/people/active-count", get(handle_active_count))
        .route("/api/ui/people/cards", get(handle_people_cards))
        .route("/api/ui/people/search", get(handle_people_search))
        .route("/api/ui/people/:id", get(handle_person_detail))
        .route("/api/ui/people/departments", get(handle_departments_list))
        .route("/api/ui/people/teams", get(handle_teams_list))
        .route("/api/ui/people/time-off", get(handle_time_off_list))
        .route("/api/ui/people/stats", get(handle_people_stats))
        .route("/api/ui/people/new", get(handle_new_person_form))
}

async fn handle_people_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PeopleQuery>,
) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = people::table
            .filter(people::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref dept) = query.department {
            db_query = db_query.filter(people::department.eq(dept));
        }

        if let Some(is_active) = query.is_active {
            db_query = db_query.filter(people::is_active.eq(is_active));
        }

        if let Some(ref search) = query.search {
            let term = format!("%{search}%");
            db_query = db_query.filter(
                people::first_name.ilike(&term)
                    .or(people::last_name.ilike(&term))
                    .or(people::email.ilike(&term))
            );
        }

        db_query = db_query.order(people::first_name.asc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                people::id,
                people::first_name,
                people::last_name,
                people::email,
                people::job_title,
                people::department,
                people::avatar_url,
                people::is_active,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(persons) if !persons.is_empty() => {
            let mut html = String::from(
                r##"<table class="people-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Email</th>
                            <th>Job Title</th>
                            <th>Department</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>"##
            );

            for (id, first_name, last_name, email, job_title, department, avatar_url, is_active) in persons {
                let full_name = format!("{} {}", first_name, last_name.unwrap_or_default());
                let email_str = email.unwrap_or_else(|| "-".to_string());
                let title_str = job_title.unwrap_or_else(|| "-".to_string());
                let dept_str = department.unwrap_or_else(|| "-".to_string());
                let avatar = avatar_url.unwrap_or_else(|| "/assets/default-avatar.png".to_string());
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    r##"<tr class="person-row" data-id="{id}">
                        <td class="person-name">
                            <img src="{}" class="avatar-sm" alt="" />
                            <span>{}</span>
                        </td>
                        <td class="person-email">{}</td>
                        <td class="person-title">{}</td>
                        <td class="person-department">{}</td>
                        <td class="person-status"><span class="{}">{}</span></td>
                        <td class="person-actions">
                            <button class="btn-sm" hx-get="/api/ui/people/{id}" hx-target="#person-detail">View</button>
                            <button class="btn-sm btn-secondary" hx-get="/api/people/{id}/edit" hx-target="#modal-content">Edit</button>
                        </td>
                    </tr>"##,
                    html_escape(&avatar),
                    html_escape(&full_name),
                    html_escape(&email_str),
                    html_escape(&title_str),
                    html_escape(&dept_str),
                    status_class,
                    status_text
                ));
            }

            html.push_str("</tbody></table>");
            Html(html)
        }
        _ => Html(
            r##"<div class="empty-state">
                <div class="empty-icon">👥</div>
                <p>No people found</p>
                <p class="empty-hint">Add people to your directory</p>
                <button class="btn btn-primary" hx-get="/api/ui/people/new" hx-target="#modal-content">Add Person</button>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_people_cards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PeopleQuery>,
) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = people::table
            .filter(people::bot_id.eq(bot_id))
            .filter(people::is_active.eq(true))
            .into_boxed();

        if let Some(ref dept) = query.department {
            db_query = db_query.filter(people::department.eq(dept));
        }

        db_query = db_query.order(people::first_name.asc());

        let limit = query.limit.unwrap_or(20);
        db_query = db_query.limit(limit);

        db_query
            .select((
                people::id,
                people::first_name,
                people::last_name,
                people::email,
                people::job_title,
                people::department,
                people::avatar_url,
                people::phone,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(persons) if !persons.is_empty() => {
            let mut html = String::from(r##"<div class="people-cards-grid">"##);

            for (id, first_name, last_name, email, job_title, department, avatar_url, phone) in persons {
                let full_name = format!("{} {}", first_name, last_name.unwrap_or_default());
                let email_str = email.unwrap_or_default();
                let title_str = job_title.unwrap_or_else(|| "Team Member".to_string());
                let dept_str = department.unwrap_or_default();
                let avatar = avatar_url.unwrap_or_else(|| "/assets/default-avatar.png".to_string());
                let phone_str = phone.unwrap_or_default();

                html.push_str(&format!(
                    r##"<div class="person-card" data-id="{id}" hx-get="/api/ui/people/{id}" hx-target="#person-detail">
                        <div class="card-avatar">
                            <img src="{}" alt="{}" class="avatar-lg" />
                        </div>
                        <div class="card-info">
                            <h4 class="card-name">{}</h4>
                            <p class="card-title">{}</p>
                            <p class="card-department">{}</p>
                        </div>
                        <div class="card-contact">
                            <span class="card-email">{}</span>
                            <span class="card-phone">{}</span>
                        </div>
                    </div>"##,
                    html_escape(&avatar),
                    html_escape(&full_name),
                    html_escape(&full_name),
                    html_escape(&title_str),
                    html_escape(&dept_str),
                    html_escape(&email_str),
                    html_escape(&phone_str)
                ));
            }

            html.push_str("</div>");
            Html(html)
        }
        _ => Html(
            r##"<div class="empty-state">
                <div class="empty-icon">👥</div>
                <p>No people in directory</p>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_people_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        people::table
            .filter(people::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_active_count(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        people::table
            .filter(people::bot_id.eq(bot_id))
            .filter(people::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_person_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        people::table
            .find(id)
            .select((
                people::id,
                people::first_name,
                people::last_name,
                people::email,
                people::phone,
                people::mobile,
                people::job_title,
                people::department,
                people::office_location,
                people::avatar_url,
                people::bio,
                people::hire_date,
                people::is_active,
                people::last_seen_at,
            ))
            .first::<(Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::NaiveDate>, bool, Option<DateTime<Utc>>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((id, first_name, last_name, email, phone, mobile, job_title, department, office, avatar_url, bio, hire_date, is_active, last_seen)) => {
            let full_name = format!("{} {}", first_name, last_name.unwrap_or_default());
            let email_str = email.unwrap_or_else(|| "-".to_string());
            let phone_str = phone.unwrap_or_else(|| "-".to_string());
            let mobile_str = mobile.unwrap_or_else(|| "-".to_string());
            let title_str = job_title.unwrap_or_else(|| "-".to_string());
            let dept_str = department.unwrap_or_else(|| "-".to_string());
            let office_str = office.unwrap_or_else(|| "-".to_string());
            let avatar = avatar_url.unwrap_or_else(|| "/assets/default-avatar.png".to_string());
            let bio_str = bio.unwrap_or_else(|| "No bio available".to_string());
            let hire_str = hire_date.map(|d| d.format("%B %d, %Y").to_string()).unwrap_or_else(|| "-".to_string());
            let last_seen_str = last_seen.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "Never".to_string());
            let status_class = if is_active { "status-active" } else { "status-inactive" };
            let status_text = if is_active { "Active" } else { "Inactive" };

            Html(format!(
                r##"<div class="person-detail-card">
                    <div class="detail-header">
                        <img src="{}" alt="{}" class="avatar-xl" />
                        <div class="header-info">
                            <h2>{}</h2>
                            <p class="job-title">{}</p>
                            <span class="{}">{}</span>
                        </div>
                    </div>
                    <div class="detail-section">
                        <h4>Contact Information</h4>
                        <div class="detail-grid">
                            <div class="detail-item">
                                <label>Email</label>
                                <span><a href="mailto:{}">{}</a></span>
                            </div>
                            <div class="detail-item">
                                <label>Phone</label>
                                <span>{}</span>
                            </div>
                            <div class="detail-item">
                                <label>Mobile</label>
                                <span>{}</span>
                            </div>
                            <div class="detail-item">
                                <label>Office</label>
                                <span>{}</span>
                            </div>
                        </div>
                    </div>
                    <div class="detail-section">
                        <h4>Work Information</h4>
                        <div class="detail-grid">
                            <div class="detail-item">
                                <label>Department</label>
                                <span>{}</span>
                            </div>
                            <div class="detail-item">
                                <label>Hire Date</label>
                                <span>{}</span>
                            </div>
                            <div class="detail-item">
                                <label>Last Seen</label>
                                <span>{}</span>
                            </div>
                        </div>
                    </div>
                    <div class="detail-section">
                        <h4>Bio</h4>
                        <p class="bio-text">{}</p>
                    </div>
                    <div class="detail-actions">
                        <button class="btn btn-primary" hx-get="/api/people/{}/edit" hx-target="#modal-content">Edit</button>
                        <button class="btn btn-secondary" hx-get="/api/ui/people/{}/reports" hx-target="#reports-panel">View Reports</button>
                        <button class="btn btn-danger" hx-delete="/api/people/{}" hx-swap="none" hx-confirm="Deactivate this person?">Deactivate</button>
                    </div>
                </div>"##,
                html_escape(&avatar),
                html_escape(&full_name),
                html_escape(&full_name),
                html_escape(&title_str),
                status_class,
                status_text,
                html_escape(&email_str),
                html_escape(&email_str),
                html_escape(&phone_str),
                html_escape(&mobile_str),
                html_escape(&office_str),
                html_escape(&dept_str),
                hire_str,
                last_seen_str,
                html_escape(&bio_str),
                id, id, id
            ))
        }
        None => Html(
            r##"<div class="empty-state">
                <p>Person not found</p>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_departments_list(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        people_departments::table
            .filter(people_departments::bot_id.eq(bot_id))
            .filter(people_departments::is_active.eq(true))
            .order(people_departments::name.asc())
            .select((
                people_departments::id,
                people_departments::name,
                people_departments::description,
                people_departments::code,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(depts) if !depts.is_empty() => {
            let mut html = String::from(r##"<div class="departments-list">"##);

            for (id, name, description, code) in depts {
                let desc_str = description.unwrap_or_default();
                let code_str = code.unwrap_or_default();

                html.push_str(&format!(
                    r##"<div class="department-item" data-id="{id}">
                        <div class="dept-header">
                            <span class="dept-name">{}</span>
                            <span class="dept-code">{}</span>
                        </div>
                        <p class="dept-description">{}</p>
                        <button class="btn-sm" hx-get="/api/ui/people?department={}" hx-target="#people-list">View Members</button>
                    </div>"##,
                    html_escape(&name),
                    html_escape(&code_str),
                    html_escape(&desc_str),
                    html_escape(&name)
                ));
            }

            html.push_str("</div>");
            Html(html)
        }
        _ => Html(
            r##"<div class="empty-state">
                <p>No departments yet</p>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_teams_list(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        people_teams::table
            .filter(people_teams::bot_id.eq(bot_id))
            .filter(people_teams::is_active.eq(true))
            .order(people_teams::name.asc())
            .select((
                people_teams::id,
                people_teams::name,
                people_teams::description,
                people_teams::color,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(teams) if !teams.is_empty() => {
            let mut html = String::from(r##"<div class="teams-list">"##);

            for (id, name, description, color) in teams {
                let desc_str = description.unwrap_or_default();
                let team_color = color.unwrap_or_else(|| "#3b82f6".to_string());

                html.push_str(&format!(
                    r##"<div class="team-item" data-id="{id}" style="border-left: 4px solid {};">
                        <div class="team-header">
                            <span class="team-name">{}</span>
                        </div>
                        <p class="team-description">{}</p>
                        <button class="btn-sm" hx-get="/api/ui/people?team_id={id}" hx-target="#people-list">View Members</button>
                    </div>"##,
                    team_color,
                    html_escape(&name),
                    html_escape(&desc_str)
                ));
            }

            html.push_str("</div>");
            Html(html)
        }
        _ => Html(
            r##"<div class="empty-state">
                <p>No teams yet</p>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_time_off_list(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        people_time_off::table
            .filter(people_time_off::bot_id.eq(bot_id))
            .filter(people_time_off::status.eq("pending"))
            .order(people_time_off::created_at.desc())
            .limit(20)
            .select((
                people_time_off::id,
                people_time_off::person_id,
                people_time_off::time_off_type,
                people_time_off::status,
                people_time_off::start_date,
                people_time_off::end_date,
                people_time_off::reason,
            ))
            .load::<(Uuid, Uuid, String, String, chrono::NaiveDate, chrono::NaiveDate, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(requests) if !requests.is_empty() => {
            let mut html = String::from(r##"<div class="time-off-list">"##);

            for (id, person_id, time_off_type, status, start_date, end_date, reason) in requests {
                let reason_str = reason.unwrap_or_default();
                let start_str = start_date.format("%b %d").to_string();
                let end_str = end_date.format("%b %d, %Y").to_string();

                html.push_str(&format!(
                    r##"<div class="time-off-item" data-id="{id}">
                        <div class="time-off-header">
                            <span class="time-off-type">{}</span>
                            <span class="time-off-status status-{}">{}</span>
                        </div>
                        <div class="time-off-dates">
                            <span>{} - {}</span>
                        </div>
                        <p class="time-off-reason">{}</p>
                        <div class="time-off-actions">
                            <button class="btn-sm btn-success" hx-put="/api/people/time-off/{id}/approve" hx-swap="none">Approve</button>
                            <button class="btn-sm btn-danger" hx-put="/api/people/time-off/{id}/reject" hx-swap="none">Reject</button>
                        </div>
                    </div>"##,
                    html_escape(&time_off_type),
                    status,
                    html_escape(&status),
                    start_str,
                    end_str,
                    html_escape(&reason_str)
                ));
            }

            html.push_str("</div>");
            Html(html)
        }
        _ => Html(
            r##"<div class="empty-state">
                <p>No pending time-off requests</p>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_people_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let q = query.q.clone().unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }

    let pool = state.conn.clone();
    let search_term = format!("%{q}%");

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        people::table
            .filter(people::bot_id.eq(bot_id))
            .filter(
                people::first_name.ilike(&search_term)
                    .or(people::last_name.ilike(&search_term))
                    .or(people::email.ilike(&search_term))
                    .or(people::job_title.ilike(&search_term))
            )
            .order(people::first_name.asc())
            .limit(20)
            .select((
                people::id,
                people::first_name,
                people::last_name,
                people::email,
                people::job_title,
                people::avatar_url,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>, Option<String>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(persons) if !persons.is_empty() => {
            let mut html = String::from(r##"<div class="search-results">"##);

            for (id, first_name, last_name, email, job_title, avatar_url) in persons {
                let full_name = format!("{} {}", first_name, last_name.unwrap_or_default());
                let email_str = email.unwrap_or_default();
                let title_str = job_title.unwrap_or_default();
                let avatar = avatar_url.unwrap_or_else(|| "/assets/default-avatar.png".to_string());

                html.push_str(&format!(
                    r##"<div class="search-result-item" hx-get="/api/ui/people/{id}" hx-target="#person-detail">
                        <img src="{}" class="avatar-sm" alt="" />
                        <div class="result-info">
                            <span class="result-name">{}</span>
                            <span class="result-title">{}</span>
                            <span class="result-email">{}</span>
                        </div>
                    </div>"##,
                    html_escape(&avatar),
                    html_escape(&full_name),
                    html_escape(&title_str),
                    html_escape(&email_str)
                ));
            }

            html.push_str("</div>");
            Html(html)
        }
        _ => Html(format!(
            r##"<div class="search-results-empty">
                <p>No results for "{}"</p>
            </div>"##,
            html_escape(&q)
        )),
    }
}

async fn handle_people_stats(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let total: i64 = people::table
            .filter(people::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let active: i64 = people::table
            .filter(people::bot_id.eq(bot_id))
            .filter(people::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let departments: i64 = people_departments::table
            .filter(people_departments::bot_id.eq(bot_id))
            .filter(people_departments::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let teams: i64 = people_teams::table
            .filter(people_teams::bot_id.eq(bot_id))
            .filter(people_teams::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        let pending_time_off: i64 = people_time_off::table
            .filter(people_time_off::bot_id.eq(bot_id))
            .filter(people_time_off::status.eq("pending"))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0);

        Some((total, active, departments, teams, pending_time_off))
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((total, active, departments, teams, pending_time_off)) => Html(format!(
            r##"<div class="stats-grid">
                <div class="stat-card">
                    <span class="stat-value">{total}</span>
                    <span class="stat-label">Total People</span>
                </div>
                <div class="stat-card stat-success">
                    <span class="stat-value">{active}</span>
                    <span class="stat-label">Active</span>
                </div>
                <div class="stat-card stat-info">
                    <span class="stat-value">{departments}</span>
                    <span class="stat-label">Departments</span>
                </div>
                <div class="stat-card stat-primary">
                    <span class="stat-value">{teams}</span>
                    <span class="stat-label">Teams</span>
                </div>
                <div class="stat-card stat-warning">
                    <span class="stat-value">{pending_time_off}</span>
                    <span class="stat-label">Pending Time Off</span>
                </div>
            </div>"##
        )),
        None => Html(r##"<div class="stats-grid"><div class="stat-card"><span class="stat-value">-</span></div></div>"##.to_string()),
    }
}

async fn handle_new_person_form() -> Html<String> {
    Html(r##"<div class="modal-header">
        <h3>Add New Person</h3>
        <button class="btn-close" onclick="closeModal()">&times;</button>
    </div>
    <form class="person-form" hx-post="/api/people" hx-swap="none">
        <div class="form-row">
            <div class="form-group">
                <label>First Name *</label>
                <input type="text" name="first_name" required />
            </div>
            <div class="form-group">
                <label>Last Name</label>
                <input type="text" name="last_name" />
            </div>
        </div>
        <div class="form-group">
            <label>Email</label>
            <input type="email" name="email" />
        </div>
        <div class="form-row">
            <div class="form-group">
                <label>Phone</label>
                <input type="tel" name="phone" />
            </div>
            <div class="form-group">
                <label>Mobile</label>
                <input type="tel" name="mobile" />
            </div>
        </div>
        <div class="form-group">
            <label>Job Title</label>
            <input type="text" name="job_title" />
        </div>
        <div class="form-group">
            <label>Department</label>
            <input type="text" name="department" />
        </div>
        <div class="form-group">
            <label>Office Location</label>
            <input type="text" name="office_location" />
        </div>
        <div class="form-group">
            <label>Hire Date</label>
            <input type="date" name="hire_date" />
        </div>
        <div class="form-group">
            <label>Bio</label>
            <textarea name="bio" rows="3" placeholder="Short biography"></textarea>
        </div>
        <div class="form-actions">
            <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
            <button type="submit" class="btn btn-primary">Add Person</button>
        </div>
    </form>"##.to_string())
}
