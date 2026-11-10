use super::*;

pub async fn get_team(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamWithMembers>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    let team: Team = scoped_team(id, branch_id)
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-team-not-found"))?;

    let member_ids: Vec<Uuid> = people_team_members::table
        .filter(people_team_members::team_id.eq(id))
        .select(people_team_members::person_id)
        .load(&mut conn)
        .unwrap_or_default();

    // Members and leader stay inside the team's branch (#2138).
    let members: Vec<Person> = if member_ids.is_empty() {
        vec![]
    } else {
        people::table
            .filter(people::branch_id.eq(branch_id))
            .filter(people::id.eq_any(&member_ids))
            .load(&mut conn)
            .unwrap_or_default()
    };

    let leader: Option<Person> = team
        .leader_id
        .and_then(|lid| scoped_person(lid, branch_id).first(&mut conn).ok());

    Ok(Json(TeamWithMembers {
        team,
        members,
        leader,
    }))
}
pub async fn add_team_member(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Path(team_id): Path<Uuid>,
    Json(req): Json<AddTeamMemberRequest>,
) -> Result<Json<TeamMember>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    // Both ends of the membership must belong to the caller's branch.
    let _team: Team = scoped_team(team_id, branch_id)
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-team-not-found"))?;
    let _person: Person = scoped_person(req.person_id, branch_id)
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-person-not-found"))?;

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
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(member))
}
pub async fn remove_team_member(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Path((team_id, person_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    let _team: Team = scoped_team(team_id, branch_id)
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-team-not-found"))?;

    diesel::delete(
        people_team_members::table
            .filter(people_team_members::team_id.eq(team_id))
            .filter(people_team_members::person_id.eq(person_id)),
    )
    .execute(&mut conn)
    .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn delete_team(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    let deleted = diesel::delete(scoped_team(id, branch_id))
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if deleted == 0 {
        return Err(api_error(
            &locale,
            StatusCode::NOT_FOUND,
            "error-team-not-found",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
pub async fn create_department(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<Department>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let (org_id, bot_id) = org_and_bot(&mut conn, branch_id);
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
        branch_id,
    };

    diesel::insert_into(people_departments::table)
        .values(&department)
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(department))
}
pub async fn list_departments(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Department>>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    let depts: Vec<Department> = people_departments::table
        .filter(people_departments::branch_id.eq(branch_id))
        .filter(people_departments::is_active.eq(true))
        .order(people_departments::name.asc())
        .load(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(depts))
}
pub async fn list_skills(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<Skill>>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    let skills: Vec<Skill> = people_skills::table
        .filter(people_skills::branch_id.eq(branch_id))
        .filter(people_skills::is_active.eq(true))
        .order(people_skills::name.asc())
        .load(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(skills))
}
pub async fn create_skill(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Json(req): Json<CreateSkillRequest>,
) -> Result<Json<Skill>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let (org_id, bot_id) = org_and_bot(&mut conn, branch_id);
    let skill = Skill {
        id,
        org_id,
        bot_id,
        name: req.name,
        category: req.category,
        description: req.description,
        is_active: true,
        created_at: now,
        branch_id,
    };

    diesel::insert_into(people_skills::table)
        .values(&skill)
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(skill))
}
pub async fn add_person_skill(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Path(person_id): Path<Uuid>,
    Json(req): Json<AddPersonSkillRequest>,
) -> Result<Json<PersonSkill>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);

    // A skill may only be attached to a person of the caller's branch.
    let _person: Person = scoped_person(person_id, branch_id)
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-person-not-found"))?;

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
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(person_skill))
}
pub async fn create_time_off(
    State(state): State<Arc<PeopleState>>,
    headers: HeaderMap,
    Json(req): Json<CreateTimeOffRequest>,
) -> Result<Json<TimeOff>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let branch_id = tenant_branch(&state, &headers, &mut conn);
    let id = Uuid::new_v4();
    let now = Utc::now();

    // A request may only be filed for a person of the caller's branch.
    let _person: Person = scoped_person(req.person_id, branch_id)
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-person-not-found"))?;

    let start_date = NaiveDate::parse_from_str(&req.start_date, "%Y-%m-%d")
        .map_err(|_| api_error(&locale, StatusCode::BAD_REQUEST, "error-people-invalid-start-date"))?;

    let end_date = NaiveDate::parse_from_str(&req.end_date, "%Y-%m-%d")
        .map_err(|_| api_error(&locale, StatusCode::BAD_REQUEST, "error-people-invalid-end-date"))?;

    let (org_id, bot_id) = org_and_bot(&mut conn, branch_id);
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
        branch_id,
    };

    diesel::insert_into(people_time_off::table)
        .values(&time_off)
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(time_off))
}
