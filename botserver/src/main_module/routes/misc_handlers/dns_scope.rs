//! Tenant scope for DNS reads and single-record edits (#2793).
//!
//! `dns_records` has no tenant column: ownership is expressed by the
//! `bot_domains` zone a hostname lives in. A caller may see a record only
//! when its hostname falls inside a zone bound to the caller's verified
//! org/branch claim or to a bot the caller manages. Admin/service principals
//! are unrestricted. Every decision is made from verified claims
//! (`AuthenticatedUser` from the auth middleware, `branch_id` from the signed
//! JWT), never from query-string or form input.

use crate::security::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use uuid::Uuid;

use super::dns_validation::hostname_in_zone;

/// Zones the caller may read, or `All` for an admin/service principal.
pub(crate) enum DnsZoneScope {
    All,
    Zones(Vec<String>),
}

/// Pure visibility predicate: true when `hostname` is the apex or a
/// sub-domain of one of the caller's zones. No I/O, so it is unit-tested with
/// two independent tenants.
pub(crate) fn hostname_visible_in(hostname: &str, scope: &DnsZoneScope) -> bool {
    match scope {
        DnsZoneScope::All => true,
        DnsZoneScope::Zones(zones) => zones.iter().any(|zone| hostname_in_zone(hostname, zone)),
    }
}

/// Resolves the zones a caller controls from verified claims only.
///
/// `branch_id` is the server-minted claim extracted from the request headers;
/// `user` is the principal installed by the auth middleware. A zone counts as
/// the caller's when its org matches the caller's org claim, its branch
/// matches the signed branch claim, or the caller manages/operates its bot.
pub(crate) fn caller_dns_scope(
    conn: &mut diesel::PgConnection,
    user: &AuthenticatedUser,
    branch_id: Option<Uuid>,
) -> Result<DnsZoneScope, String> {
    if user.is_admin() || user.is_service() {
        return Ok(DnsZoneScope::All);
    }

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ZoneRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        domain: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        org_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        branch_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
    }

    let zones: Vec<ZoneRow> =
        diesel::sql_query("SELECT domain, org_id, branch_id, bot_id FROM bot_domains")
            .load(conn)
            .map_err(|e| format!("Zone ownership lookup failed: {e}"))?;

    let mut allowed = Vec::new();
    for zone in zones {
        let org_ok = zone
            .org_id
            .is_some_and(|org| user.can_access_organization(&org));
        let branch_ok = branch_id.is_some_and(|caller| zone.branch_id == Some(caller));
        let bot_ok = user.can_manage_bot(&zone.bot_id) || user.can_operate_bot(&zone.bot_id);
        if org_ok || branch_ok || bot_ok {
            allowed.push(zone.domain);
        }
    }
    Ok(DnsZoneScope::Zones(allowed))
}

/// Single-record authorization gate: the record's hostname must live in a
/// zone covered by the caller's verified org/branch/bot claims.
pub(crate) fn caller_owns_hostname_in(
    conn: &mut diesel::PgConnection,
    user: &AuthenticatedUser,
    branch_id: Option<Uuid>,
    hostname: &str,
) -> Result<bool, String> {
    let scope = caller_dns_scope(conn, user, branch_id)?;
    Ok(hostname_visible_in(hostname, &scope))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(zones: &[&str]) -> DnsZoneScope {
        DnsZoneScope::Zones(zones.iter().map(|z| (*z).to_string()).collect())
    }

    #[test]
    fn two_tenants_only_see_their_own_hostnames() {
        let tenant_a = scope(&["a.example.com"]);
        assert!(hostname_visible_in("www.a.example.com", &tenant_a));
        assert!(hostname_visible_in("a.example.com", &tenant_a));
        // Tenant B's records are foreign: filtered from list/search and 404
        // when requested through the edit fragment.
        assert!(!hostname_visible_in("www.b.example.com", &tenant_a));
        assert!(!hostname_visible_in("b.example.com", &tenant_a));
        // Label-boundary lookalikes must not leak either.
        assert!(!hostname_visible_in("nota.example.com", &tenant_a));
        assert!(!hostname_visible_in("a.example.com.evil.org", &tenant_a));
    }

    #[test]
    fn each_tenant_sees_only_its_own_zone_from_a_two_tenant_catalog() {
        let tenant_a = scope(&["a.example.com"]);
        let tenant_b = scope(&["b.example.com"]);
        assert!(hostname_visible_in("mail.a.example.com", &tenant_a));
        assert!(!hostname_visible_in("mail.b.example.com", &tenant_a));
        assert!(hostname_visible_in("mail.b.example.com", &tenant_b));
        assert!(!hostname_visible_in("mail.a.example.com", &tenant_b));
    }

    #[test]
    fn unrestricted_sees_all_and_empty_scope_sees_none() {
        assert!(hostname_visible_in("x.c.example.com", &DnsZoneScope::All));
        assert!(!hostname_visible_in("x.c.example.com", &scope(&[])));
        assert!(!hostname_visible_in("", &scope(&["a.example.com"])));
    }

    #[test]
    fn zone_matching_ignores_case_and_trailing_dot() {
        assert!(hostname_visible_in(
            "WWW.A.Example.COM.",
            &scope(&["a.example.com"])
        ));
    }
}
