use super::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecipientsQuery {
    pub query: Option<String>,
}

/// Candidate account returned by the recipient query, together with every org
/// it belongs to. The org ids are collected in SQL and the tenant boundary is
/// enforced by `shares_org`, which is shared with the tests below.
#[derive(QueryableByName)]
struct CandidateRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    email: String,
    #[diesel(sql_type = Text)]
    username: String,
    #[diesel(sql_type = diesel::sql_types::Array<Text>)]
    org_ids: Vec<String>,
    #[diesel(sql_type = diesel::sql_types::Array<Text>)]
    contact_org_ids: Vec<String>,
}

#[derive(QueryableByName)]
struct OrgRow {
    #[diesel(sql_type = Text)]
    org_id: String,
}

/// #1639 — recipient picker for "Share with person". Returns at most 10 active
/// accounts whose email or username contains the typed text AND that share an
/// organization with the caller. The caller still has to choose one, and the
/// share row is written by `share_folder`.
pub async fn list_share_recipients(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<RecipientsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    // #2670 — the caller's org membership is the tenant boundary. Never
    // discard the identity: a global user search is a cross-tenant user
    // enumeration primitive.
    let caller_id = user.user_id;
    let caller_email = user.email.as_deref().unwrap_or("");
    let query = params.query.unwrap_or_else(String::new).trim().to_lowercase();
    if query.len() < 2 {
        return Ok(Json(Vec::new()));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // Resolve the caller's orgs with the same signup-derived binding used by
    // cached_quota::allowed_buckets_for: explicit user_organizations rows plus
    // the crm_contacts -> branches -> organizations mapping created on signup.
    // A DB error is fatal here — an empty/failed lookup must never widen the
    // search to the whole `users` table.
    let allowed_orgs: Vec<String> = match diesel::sql_query(
        "SELECT org_id::text AS org_id FROM user_organizations WHERE user_id = $1 \
         UNION \
         SELECT uo.org_id::text AS org_id FROM user_organizations uo \
           JOIN users c ON c.id = uo.user_id \
           WHERE lower(c.email) = lower($2) \
         UNION \
         SELECT br.org_id::text AS org_id FROM crm_contacts c \
           JOIN branches br ON br.id = c.branch_id \
           WHERE lower(c.email) = lower($2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(caller_id)
    .bind::<Text, _>(caller_email)
    .load::<OrgRow>(&mut conn)
    {
        Ok(rows) => rows.into_iter().map(|r| r.org_id).collect(),
        Err(e) => {
            warn!("drive: share-recipient org lookup failed for {caller_id}: {e}");
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Recipient lookup unavailable",
            ));
        }
    };

    // Fail closed: a caller with no org membership sees nobody.
    if allowed_orgs.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // Fetch every active text match with its org memberships and apply the
    // tenant predicate in Rust so the boundary is a single, testable function.
    // The candidate query is intentionally not truncated at SQL level: a
    // foreign tenant must not crowd out same-org users before the filter runs.
    let candidates = match diesel::sql_query(
        "SELECT u.id::text AS id, u.email, u.username, \
                COALESCE(array_agg(DISTINCT uo.org_id::text) \
                  FILTER (WHERE uo.org_id IS NOT NULL), '{}'::text[]) AS org_ids, \
                COALESCE(array_agg(DISTINCT br.org_id::text) \
                  FILTER (WHERE br.org_id IS NOT NULL), '{}'::text[]) AS contact_org_ids \
         FROM users u \
         LEFT JOIN user_organizations uo ON uo.user_id = u.id \
         LEFT JOIN crm_contacts c ON lower(c.email) = lower(u.email) \
         LEFT JOIN branches br ON br.id = c.branch_id \
         WHERE u.is_active IS NOT FALSE \
           AND (LOWER(u.email) LIKE $1 OR LOWER(u.username) LIKE $1) \
         GROUP BY u.id, u.email, u.username \
         ORDER BY u.email",
    )
    .bind::<Text, _>(format!("%{query}%"))
    .load::<CandidateRow>(&mut conn)
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("drive: share-recipient candidate lookup failed: {e}");
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Recipient lookup unavailable",
            ));
        }
    };

    Ok(Json(visible_recipients(candidates, &allowed_orgs)))
}

/// Filter candidates to those sharing an org with the caller and cap the
/// picker at 10 results. This is the exact production path exercised by the
/// two-tenant test below.
fn visible_recipients(
    candidates: Vec<CandidateRow>,
    allowed_orgs: &[String],
) -> Vec<serde_json::Value> {
    candidates
        .into_iter()
        .filter(|row| shares_org(&row.org_ids, &row.contact_org_ids, allowed_orgs))
        .take(10)
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "email": row.email,
                "username": row.username,
            })
        })
        .collect()
}

/// #2670 — the tenant predicate. A candidate is visible only when it shares
/// at least one org with the caller, through `user_organizations` or through
/// the signup-derived `crm_contacts -> branches` binding. Pure so the tenant
/// boundary is testable with two tenants without a database.
fn shares_org(org_ids: &[String], contact_org_ids: &[String], allowed_orgs: &[String]) -> bool {
    org_ids
        .iter()
        .chain(contact_org_ids.iter())
        .any(|org| allowed_orgs.contains(org))
}

#[cfg(test)]
mod recipient_scope_tests {
    use super::*;

    fn candidate(email: &str, org_ids: &[&str], contact_org_ids: &[&str]) -> CandidateRow {
        CandidateRow {
            id: format!("id-{email}"),
            email: email.to_string(),
            username: email.split('@').next().unwrap_or("").to_string(),
            org_ids: org_ids.iter().map(|s| s.to_string()).collect(),
            contact_org_ids: contact_org_ids.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// #2670 — two tenants: the caller belongs to `org-a`, a second user is
    /// only in `org-b`, and a third is only bound to `org-b` via a CRM contact.
    /// Only the same-org user may appear in the picker.
    #[test]
    fn foreign_tenant_users_are_not_returned() {
        let allowed_orgs = vec!["org-a".to_string()];
        let candidates = vec![
            candidate("alice@tenant-a.example", &["org-a"], &[]),
            candidate("bob@tenant-b.example", &["org-b"], &[]),
            candidate("carol@tenant-b.example", &[], &["org-b"]),
        ];

        let visible = visible_recipients(candidates, &allowed_orgs);
        let emails: Vec<&str> = visible
            .iter()
            .map(|v| v["email"].as_str().unwrap_or(""))
            .collect();

        assert_eq!(emails, vec!["alice@tenant-a.example"]);
        assert!(
            !emails.contains(&"bob@tenant-b.example"),
            "user_organizations foreign tenant must be excluded"
        );
        assert!(
            !emails.contains(&"carol@tenant-b.example"),
            "crm_contacts foreign tenant must be excluded"
        );
    }

    #[test]
    fn shared_member_in_both_orgs_is_visible() {
        let allowed_orgs = vec!["org-a".to_string()];
        let candidates = vec![
            candidate("shared@tenant-ab.example", &["org-a", "org-b"], &[]),
            candidate("bob@tenant-b.example", &["org-b"], &[]),
        ];

        let visible = visible_recipients(candidates, &allowed_orgs);
        assert_eq!(visible.len(), 1);
        assert_eq!(
            visible[0]["email"].as_str().unwrap_or(""),
            "shared@tenant-ab.example"
        );
    }

    #[test]
    fn no_caller_orgs_returns_no_recipients() {
        let candidates = vec![candidate("alice@tenant-a.example", &["org-a"], &[])];
        assert!(visible_recipients(candidates, &[]).is_empty());
    }

    /// Guard the SQL half of the boundary: both org sources must be collected
    /// or the predicate above would silently exclude contact-bound users.
    #[test]
    fn candidate_query_collects_both_org_sources() {
        let sql = include_str!("share_recipients.rs");
        assert!(sql.contains("JOIN user_organizations uo ON uo.user_id = u.id"));
        assert!(sql.contains("JOIN crm_contacts c ON lower(c.email) = lower(u.email)"));
        assert!(sql.contains("JOIN branches br ON br.id = c.branch_id"));
        assert!(
            !sql.contains("let _ = user"),
            "the caller identity must not be discarded"
        );
    }
}
