pub mod ui;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};

use bigdecimal::BigDecimal;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{
    people_departments, people_org_chart, people_person_skills, people_skills,
    people_team_members, people_teams, people_time_off,
};
use crate::core::shared::schema::people::people as people_table;
use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = people_table)]
pub struct Person {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
    pub office_location: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub birthday: Option<NaiveDate>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub skills: Vec<String>,
    pub social_links: serde_json::Value,
    pub custom_fields: serde_json::Value,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = people_teams)]
pub struct Team {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub leader_id: Option<Uuid>,
    pub parent_team_id: Option<Uuid>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = people_team_members)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub person_id: Uuid,
    pub role: Option<String>,
    pub is_primary: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = people_org_chart)]
pub struct OrgChartEntry {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub person_id: Uuid,
    pub reports_to_id: Option<Uuid>,
    pub position_title: Option<String>,
    pub position_level: i32,
    pub position_order: i32,
    pub effective_from: Option<NaiveDate>,
    pub effective_until: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = people_departments)]
pub struct Department {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<Uuid>,
    pub head_id: Option<Uuid>,
    pub cost_center: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = people_skills)]
pub struct Skill {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = people_person_skills)]
pub struct PersonSkill {
    pub id: Uuid,
    pub person_id: Uuid,
    pub skill_id: Uuid,
    pub proficiency_level: i32,
    pub years_experience: Option<BigDecimal>,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = people_time_off)]
pub struct TimeOff {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub person_id: Uuid,
    pub time_off_type: String,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub hours_requested: Option<BigDecimal>,
    pub reason: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePersonRequest {
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
    pub office_location: Option<String>,
    pub hire_date: Option<String>,
    pub birthday: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub skills: Option<Vec<String>>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
    pub office_location: Option<String>,
    pub bio: Option<String>,
    pub skills: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub leader_id: Option<Uuid>,
    pub parent_team_id: Option<Uuid>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddTeamMemberRequest {
    pub person_id: Uuid,
    pub role: Option<String>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<Uuid>,
    pub head_id: Option<Uuid>,
    pub cost_center: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddPersonSkillRequest {
    pub skill_id: Uuid,
    pub proficiency_level: Option<i32>,
    pub years_experience: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTimeOffRequest {
    pub person_id: Uuid,
    pub time_off_type: String,
    pub start_date: String,
    pub end_date: String,
    pub hours_requested: Option<f64>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveTimeOffRequest {
    pub approved: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub department: Option<String>,
    pub team_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PeopleStats {
    pub total_people: i64,
    pub active_people: i64,
    pub total_teams: i64,
    pub total_departments: i64,
    pub pending_time_off: i64,
    pub new_hires_this_month: i64,
}

#[derive(Debug, Serialize)]
pub struct PersonWithDetails {
    pub person: Person,
    pub manager: Option<Person>,
    pub direct_reports: Vec<Person>,
    pub teams: Vec<Team>,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Serialize)]
pub struct TeamWithMembers {
    pub team: Team,
    pub members: Vec<Person>,
    pub leader: Option<Person>,
}

#[derive(Debug, Serialize)]
pub struct OrgChartNode {
    pub person: Person,
    pub position_title: Option<String>,
    pub position_level: i32,
    pub reports: Vec<OrgChartNode>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn bd(val: f64) -> BigDecimal {
    use std::str::FromStr;
    BigDecimal::from_str(&val.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
}

pub async fn create_person(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePersonRequest>,
) -> Result<Json<Person>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let hire_date = req
        .hire_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let birthday = req
        .birthday
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let person = Person {
        id,
        org_id,
        bot_id,
        user_id: None,
        first_name: req.first_name,
        last_name: req.last_name,
        email: req.email,
        phone: req.phone,
        mobile: req.mobile,
        job_title: req.job_title,
        department: req.department,
        manager_id: req.manager_id,
        office_location: req.office_location,
        hire_date,
        birthday,
        avatar_url: req.avatar_url,
        bio: req.bio,
        skills: req.skills.unwrap_or_default(),
        social_links: serde_json::json!({}),
        custom_fields: serde_json::json!({}),
        timezone: req.timezone,
        locale: req.locale,
        is_active: true,
        last_seen_at: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(people_table::table)
        .values(&person)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(person))
}

pub async fn list_people(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Person>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = people_table::table
        .filter(people_table::org_id.eq(org_id))
        .filter(people_table::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_active) = query.is_active {
        q = q.filter(people_table::is_active.eq(is_active));
    }

    if let Some(department) = query.department {
        q = q.filter(people_table::department.eq(department));
    }

    if let Some(manager_id) = query.manager_id {
        q = q.filter(people_table::manager_id.eq(manager_id));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            people_table::first_name
                .ilike(pattern.clone())
                .or(people_table::last_name.ilike(pattern.clone()))
                .or(people_table::email.ilike(pattern.clone()))
                .or(people_table::job_title.ilike(pattern)),
        );
    }

    let persons: Vec<Person> = q
        .order((people_table::last_name.asc(), people_table::first_name.asc()))
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(persons))
}

pub async fn get_person(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonWithDetails>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let person: Person = people_table::table
        .filter(people_table::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Person not found".to_string()))?;

    let manager: Option<Person> = person
        .manager_id
        .and_then(|mid| {
            people_table::table
                .filter(people_table::id.eq(mid))
                .first(&mut conn)
                .ok()
        });

    let direct_reports: Vec<Person> = people_table::table
        .filter(people_table::manager_id.eq(id))
        .filter(people_table::is_active.eq(true))
        .order(people_table::first_name.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let team_ids: Vec<Uuid> = people_team_members::table
        .filter(people_team_members::person_id.eq(id))
        .select(people_team_members::team_id)
        .load(&mut conn)
        .unwrap_or_default();

    let teams: Vec<Team> = if team_ids.is_empty() {
        vec![]
    } else {
        people_teams::table
            .filter(people_teams::id.eq_any(&team_ids))
            .load(&mut conn)
            .unwrap_or_default()
    };

    let skill_ids: Vec<Uuid> = people_person_skills::table
        .filter(people_person_skills::person_id.eq(id))
        .select(people_person_skills::skill_id)
        .load(&mut conn)
        .unwrap_or_default();

    let skills: Vec<Skill> = if skill_ids.is_empty() {
        vec![]
    } else {
        people_skills::table
            .filter(people_skills::id.eq_any(&skill_ids))
            .load(&mut conn)
            .unwrap_or_default()
    };

    Ok(Json(PersonWithDetails {
        person,
        manager,
        direct_reports,
        teams,
        skills,
    }))
}

pub async fn update_person(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePersonRequest>,
) -> Result<Json<Person>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(people_table::table.filter(people_table::id.eq(id)))
        .set(people_table::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(first_name) = req.first_name {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::first_name.eq(first_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(last_name) = req.last_name {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::last_name.eq(last_name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(email) = req.email {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::email.eq(email))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(job_title) = req.job_title {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::job_title.eq(job_title))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(department) = req.department {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::department.eq(department))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(manager_id) = req.manager_id {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::manager_id.eq(Some(manager_id)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(is_active) = req.is_active {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::is_active.eq(is_active))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(skills) = req.skills {
        diesel::update(people_table::table.filter(people_table::id.eq(id)))
            .set(people_table::skills.eq(skills))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    let person: Person = people_table::table
        .filter(people_table::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Person not found".to_string()))?;

    Ok(Json(person))
}

pub async fn delete_person(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(people_table::table.filter(people_table::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_direct_reports(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Person>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let reports: Vec<Person> = people_table::table
        .filter(people_table::manager_id.eq(id))
        .filter(people_table::is_active.eq(true))
        .order(people_table::first_name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(reports))
}

pub async fn create_team(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<Team>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let team = Team {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        leader_id: req.leader_id,
        parent_team_id: req.parent_team_id,
        color: req.color,
        icon: req.icon,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(people_teams::table)
        .values(&team)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(team))
}

pub async fn list_teams(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Team>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let teams: Vec<Team> = people_teams::table
        .filter(people_teams::org_id.eq(org_id))
        .filter(people_teams::bot_id.eq(bot_id))
        .filter(people_teams::is_active.eq(true))
        .order(people_teams::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(teams))
}

pub async fn get_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamWithMembers>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let team: Team = people_teams::table
        .filter(people_teams::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Team not found".to_string()))?;

    let member_ids: Vec<Uuid> = people_team_members::table
        .filter(people_team_members::team_id.eq(id))
        .select(people_team_members::person_id)
        .load(&mut conn)
        .unwrap_or_default();

    let members: Vec<Person> = if member_ids.is_empty() {
        vec![]
    } else {
        people_table::table
            .filter(people_table::id.eq_any(&member_ids))
            .load(&mut conn)
            .unwrap_or_default()
    };

    let leader: Option<Person> = team
        .leader_id
        .and_then(|lid| {
            people_table::table
                .filter(people_table::id.eq(lid))
                .first(&mut conn)
                .ok()
        });

    Ok(Json(TeamWithMembers {
        team,
        members,
        leader,
    }))
}

pub async fn add_team_member(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AddTeamMemberRequest>,
) -> Result<Json<TeamMember>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let member = TeamMember {
        id,
        team_id,
        person_id: req.person_id,
        role: req.role,
        is_primary: req.is_primary.unwrap_or(false),
        joined_at: now,
    };

    diesel::insert_into(people_team_members::table)
        .values(&member)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(member))
}

pub async fn remove_team_member(
    State(state): State<Arc<AppState>>,
    Path((team_id, person_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(
        people_team_members::table
            .filter(people_team_members::team_id.eq(team_id))
            .filter(people_team_members::person_id.eq(person_id)),
    )
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(people_teams::table.filter(people_teams::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_department(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<Department>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let department = Department {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        code: req.code,
        parent_id: req.parent_id,
        head_id: req.head_id,
        cost_center: req.cost_center,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(people_departments::table)
        .values(&department)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(department))
}

pub async fn list_departments(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Department>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let depts: Vec<Department> = people_departments::table
        .filter(people_departments::org_id.eq(org_id))
        .filter(people_departments::bot_id.eq(bot_id))
        .filter(people_departments::is_active.eq(true))
        .order(people_departments::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(depts))
}

pub async fn list_skills(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Skill>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let skills: Vec<Skill> = people_skills::table
        .filter(people_skills::org_id.eq(org_id))
        .filter(people_skills::bot_id.eq(bot_id))
        .filter(people_skills::is_active.eq(true))
        .order(people_skills::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(skills))
}

pub async fn create_skill(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSkillRequest>,
) -> Result<Json<Skill>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let skill = Skill {
        id,
        org_id,
        bot_id,
        name: req.name,
        category: req.category,
        description: req.description,
        is_active: true,
        created_at: now,
    };

    diesel::insert_into(people_skills::table)
        .values(&skill)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(skill))
}

pub async fn add_person_skill(
    State(state): State<Arc<AppState>>,
    Path(person_id): Path<Uuid>,
    Json(req): Json<AddPersonSkillRequest>,
) -> Result<Json<PersonSkill>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let person_skill = PersonSkill {
        id,
        person_id,
        skill_id: req.skill_id,
        proficiency_level: req.proficiency_level.unwrap_or(1),
        years_experience: req.years_experience.map(bd),
        verified_by: None,
        verified_at: None,
        created_at: now,
    };

    diesel::insert_into(people_person_skills::table)
        .values(&person_skill)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(person_skill))
}

pub async fn create_time_off(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTimeOffRequest>,
) -> Result<Json<TimeOff>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let start_date = NaiveDate::parse_from_str(&req.start_date, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid start_date".to_string()))?;

    let end_date = NaiveDate::parse_from_str(&req.end_date, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid end_date".to_string()))?;

    let time_off = TimeOff {
        id,
        org_id,
        bot_id,
        person_id: req.person_id,
        time_off_type: req.time_off_type,
        status: "pending".to_string(),
        start_date,
        end_date,
        hours_requested: req.hours_requested.map(bd),
        reason: req.reason,
        approved_by: None,
        approved_at: None,
        notes: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(people_time_off::table)
        .values(&time_off)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(time_off))
}

pub async fn list_time_off(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<TimeOff>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let time_offs: Vec<TimeOff> = people_time_off::table
        .filter(people_time_off::org_id.eq(org_id))
        .filter(people_time_off::bot_id.eq(bot_id))
        .order(people_time_off::start_date.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(time_offs))
}

pub async fn approve_time_off(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveTimeOffRequest>,
) -> Result<Json<TimeOff>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();
    let status = if req.approved { "approved" } else { "rejected" };

    diesel::update(people_time_off::table.filter(people_time_off::id.eq(id)))
        .set((
            people_time_off::status.eq(status),
            people_time_off::approved_at.eq(Some(now)),
            people_time_off::notes.eq(req.notes),
            people_time_off::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let time_off: TimeOff = people_time_off::table
        .filter(people_time_off::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Time off request not found".to_string()))?;

    Ok(Json(time_off))
}

pub async fn get_people_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PeopleStats>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let total_people: i64 = people_table::table
        .filter(people_table::org_id.eq(org_id))
        .filter(people_table::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let active_people: i64 = people_table::table
        .filter(people_table::org_id.eq(org_id))
        .filter(people_table::bot_id.eq(bot_id))
        .filter(people_table::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_teams: i64 = people_teams::table
        .filter(people_teams::org_id.eq(org_id))
        .filter(people_teams::bot_id.eq(bot_id))
        .filter(people_teams::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_departments: i64 = people_departments::table
        .filter(people_departments::org_id.eq(org_id))
        .filter(people_departments::bot_id.eq(bot_id))
        .filter(people_departments::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let pending_time_off: i64 = people_time_off::table
        .filter(people_time_off::org_id.eq(org_id))
        .filter(people_time_off::bot_id.eq(bot_id))
        .filter(people_time_off::status.eq("pending"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let today = Utc::now().date_naive();
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .unwrap_or(today);

    let new_hires_this_month: i64 = people_table::table
        .filter(people_table::org_id.eq(org_id))
        .filter(people_table::bot_id.eq(bot_id))
        .filter(people_table::hire_date.ge(month_start))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let stats = PeopleStats {
        total_people,
        active_people,
        total_teams,
        total_departments,
        pending_time_off,
        new_hires_this_month,
    };

    Ok(Json(stats))
}

pub fn configure_people_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/people", get(list_people).post(create_person))
        .route("/api/people/stats", get(get_people_stats))
        .route("/api/people/:id", get(get_person).put(update_person).delete(delete_person))
        .route("/api/people/:id/reports", get(get_direct_reports))
        .route("/api/people/:id/skills", post(add_person_skill))
        .route("/api/people/teams", get(list_teams).post(create_team))
        .route("/api/people/teams/:id", get(get_team).delete(delete_team))
        .route("/api/people/teams/:id/members", post(add_team_member))
        .route("/api/people/teams/:team_id/members/:person_id", delete(remove_team_member))
        .route("/api/people/departments", get(list_departments).post(create_department))
        .route("/api/people/skills", get(list_skills).post(create_skill))
        .route("/api/people/time-off", get(list_time_off).post(create_time_off))
        .route("/api/people/time-off/:id/approve", put(approve_time_off))
}
