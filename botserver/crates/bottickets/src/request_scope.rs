//! Branch scope for every ticket query (#2138).
//!
//! Ticket, comment, activity and ITSM by-id endpoints used to filter on the
//! primary key alone, so any authenticated caller holding a UUID from another
//! tenant could read, rewrite, delete or comment on it. Every ticket query now
//! resolves its branch here — the verified JWT/session binding when the caller
//! has one, the crate default bot branch otherwise — and addresses a single
//! row through [`scoped_ticket`], which always carries both `id` and
//! `branch_id`, so a foreign id is simply "not found".

use super::*;

/// Branch that owns this caller's tickets.
pub(super) fn tenant_branch(
    state: &Arc<TicketsState>,
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Uuid {
    crate::scope::branch_from_jwt(headers, conn).unwrap_or_else(|| get_bot_context(state))
}

/// `support_tickets` addressed by primary key *and* tenant branch.
pub(super) type ScopedTicket = diesel::dsl::Filter<
    diesel::dsl::Filter<support_tickets::table, diesel::dsl::Eq<support_tickets::id, Uuid>>,
    diesel::dsl::Eq<support_tickets::branch_id, Uuid>,
>;

/// The only accepted way to address one ticket.
pub(super) fn scoped_ticket(id: Uuid, branch_id: Uuid) -> ScopedTicket {
    support_tickets::table
        .filter(support_tickets::id.eq(id))
        .filter(support_tickets::branch_id.eq(branch_id))
}
