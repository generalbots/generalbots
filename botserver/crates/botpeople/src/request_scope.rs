//! Branch scope for every CRM by-id query (#2138).
//!
//! People, team and time-off endpoints used to filter on the primary key
//! alone, so any authenticated caller holding a UUID from another tenant could
//! read, rewrite or delete it. Every query now resolves its branch here — the
//! verified JWT/session binding when the caller has one, the crate default bot
//! branch otherwise — and addresses a single row through `scoped_*`, which
//! always carries both `id` and `branch_id`, so a foreign id is "not found".

use super::*;

/// Branch that owns this caller's people data.
pub(super) fn tenant_branch(
    state: &PeopleState,
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Uuid {
    crate::scope::branch_from_jwt(headers, conn).unwrap_or_else(|| get_bot_context(state))
}

/// `people` addressed by primary key *and* tenant branch.
pub(super) type ScopedPerson = diesel::dsl::Filter<
    diesel::dsl::Filter<people::table, diesel::dsl::Eq<people::id, Uuid>>,
    diesel::dsl::Eq<people::branch_id, Uuid>,
>;

/// The only accepted way to address one person.
pub(super) fn scoped_person(id: Uuid, branch_id: Uuid) -> ScopedPerson {
    people::table
        .filter(people::id.eq(id))
        .filter(people::branch_id.eq(branch_id))
}

/// `people_teams` addressed by primary key *and* tenant branch.
pub(super) type ScopedTeam = diesel::dsl::Filter<
    diesel::dsl::Filter<people_teams::table, diesel::dsl::Eq<people_teams::id, Uuid>>,
    diesel::dsl::Eq<people_teams::branch_id, Uuid>,
>;

/// The only accepted way to address one team.
pub(super) fn scoped_team(id: Uuid, branch_id: Uuid) -> ScopedTeam {
    people_teams::table
        .filter(people_teams::id.eq(id))
        .filter(people_teams::branch_id.eq(branch_id))
}

/// `people_time_off` addressed by primary key *and* tenant branch.
pub(super) type ScopedTimeOff = diesel::dsl::Filter<
    diesel::dsl::Filter<people_time_off::table, diesel::dsl::Eq<people_time_off::id, Uuid>>,
    diesel::dsl::Eq<people_time_off::branch_id, Uuid>,
>;

/// The only accepted way to address one time-off request.
pub(super) fn scoped_time_off(id: Uuid, branch_id: Uuid) -> ScopedTimeOff {
    people_time_off::table
        .filter(people_time_off::id.eq(id))
        .filter(people_time_off::branch_id.eq(branch_id))
}
