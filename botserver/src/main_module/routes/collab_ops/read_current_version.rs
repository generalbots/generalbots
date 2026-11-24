use super::*;

/// Server-selected maximum number of ops returned by a single `list_ops` page.
/// A caller may request fewer via `?limit=`, but never more — this is the hard
/// ceiling that keeps a `since=0` request from loading the whole log.
pub(crate) const OPS_PAGE_LIMIT: i64 = 1000;

/// Server-selected cap (in bytes) on a single op's `payload` in the response.
/// Oversized or malformed payloads are replaced with `null` rather than
/// streamed to the caller.
pub(crate) const OPS_PAYLOAD_BYTE_CAP: usize = 64 * 1024;

/// Raw SQL for `list_ops`. Kept as a constant so an inline test can prove the
/// `LIMIT` guard cannot be dropped by accident.
const LIST_OPS_SQL: &str = "SELECT id::text, actor_id, actor_name, actor_type, op_type, \
                            base_version, lamport_ts, payload, conflict, resolved, resolution, \
                            created_at \
                            FROM collab_ops \
                            WHERE resource_type = $1 AND resource_id = $2 AND lamport_ts > $3 \
                            ORDER BY lamport_ts ASC \
                            LIMIT $4";

/// Clamp a caller-supplied page size into `[1, OPS_PAGE_LIMIT]`. A missing
/// `limit` uses the server default; hostile values (zero, negative, or huge)
/// are clamped, so an unbounded read is impossible.
fn clamp_ops_limit(requested: Option<i64>) -> i64 {
    requested.unwrap_or(OPS_PAGE_LIMIT).clamp(1, OPS_PAGE_LIMIT)
}

/// Parse an op payload into JSON, replacing oversized or malformed payloads
/// with `null` so a single op cannot blow up the response size. Truncation is
/// done on a UTF-8 character boundary to avoid slicing a code point in half.
fn capped_payload(raw: &str) -> serde_json::Value {
    let bounded = if raw.len() > OPS_PAYLOAD_BYTE_CAP {
        let mut end = OPS_PAYLOAD_BYTE_CAP;
        while end > 0 && !raw.is_char_boundary(end) {
            end -= 1;
        }
        &raw[..end]
    } else {
        raw
    };
    serde_json::from_str(bounded).unwrap_or(serde_json::Value::Null)
}

pub(super) fn read_current_version(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = BigInt)]
        current_version: i64,
    }
    let opt = diesel::sql_query(
        "SELECT current_version FROM collab_resource_state \
         WHERE resource_type = $1 AND resource_id = $2",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .get_result::<Row>(conn)
    .optional()?;
    Ok(opt.map(|r| r.current_version).unwrap_or(0))
}
pub(super) fn read_vector(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
) -> Result<HashMap<String, i64>, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        version_vector: String,
    }
    let opt = diesel::sql_query(
        "SELECT version_vector FROM collab_resource_state \
         WHERE resource_type = $1 AND resource_id = $2",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .get_result::<Row>(conn)
    .optional()?;
    Ok(opt
        .and_then(|r| serde_json::from_str::<HashMap<String, i64>>(&r.version_vector).ok())
        .map_or_else(Default::default, |v| v))
}
pub(super) fn upsert_state(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
    current_version: i64,
    vector_json: &str,
) -> Result<usize, diesel::result::Error> {
    diesel::sql_query(
        "INSERT INTO collab_resource_state \
         (resource_type, resource_id, current_version, version_vector, updated_at) \
         VALUES ($1, $2, $3, $4, NOW()) \
         ON CONFLICT (resource_type, resource_id) \
         DO UPDATE SET current_version = $3, version_vector = $4, updated_at = NOW()",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .bind::<BigInt, _>(current_version)
    .bind::<Text, _>(vector_json)
    .execute(conn)
}
/// The collaborative operation to append to the log.
pub(super) struct NewOp<'a> {
    pub conn: &'a mut PgConnection,
    pub ty: &'a str,
    pub id: &'a str,
    pub actor_id: &'a str,
    pub actor_name: &'a str,
    pub actor_type: &'a str,
    pub op_type: &'a str,
    pub base_version: i64,
    pub lamport: i64,
    pub payload_json: &'a str,
    pub conflict: bool,
}

pub(super) fn insert_op(op: NewOp<'_>) -> Result<String, diesel::result::Error> {
    let NewOp {
        conn,
        ty,
        id,
        actor_id,
        actor_name,
        actor_type,
        op_type,
        base_version,
        lamport,
        payload_json,
        conflict,
    } = op;
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
    }
    diesel::sql_query(
        "INSERT INTO collab_ops \
         (resource_type, resource_id, actor_id, actor_name, actor_type, op_type, \
          base_version, lamport_ts, payload, conflict) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         RETURNING id::text",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .bind::<Text, _>(actor_id)
    .bind::<Text, _>(actor_name)
    .bind::<Text, _>(actor_type)
    .bind::<Text, _>(op_type)
    .bind::<BigInt, _>(base_version)
    .bind::<BigInt, _>(lamport)
    .bind::<Text, _>(payload_json)
    .bind::<Bool, _>(conflict)
    .get_result::<Row>(conn)
    .map(|r| r.id)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------
/// `POST /api/collab/ops` — submit an operation. Returns the op id, its Lamport
/// timestamp, the new converged version, and whether it conflicted.
pub async fn submit_op(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(body): Json<SubmitOpBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&body.resource_type, &body.resource_id) {
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-resource"));
    }
    if body.base_version < 0 {
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-base-version"));
    }
    if body.op_type.trim().is_empty() || body.op_type.len() > 32 {
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-op-type"));
    }

    let ty = body.resource_type;
    let id = body.resource_id;
    let op_type = body.op_type;
    let base_version = body.base_version;
    let actor_type = normalize_actor_type(&body.actor_type);
    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let payload_json = serde_json::to_string(&body.payload).unwrap_or_else(|_| "{}".to_string());

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // #2731 — the resource type/id are only a lookup key. Access is decided
    // from the caller's verified claims against persisted grants, never from
    // the request body, so a foreign tenant cannot append to another's log.
    if !has_collab_access(&mut conn, &user, &ty, &id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
    }

    let outcome: Result<serde_json::Value, diesel::result::Error> = conn.transaction(|tx| {
        let current = read_current_version(tx, &ty, &id)?;
        let conflict = base_version < current;
        let lamport = current.max(base_version) + 1;

        let op_id = insert_op(NewOp {
            conn: tx,
            ty: &ty,
            id: &id,
            actor_id: &actor_id,
            actor_name: &actor_name,
            actor_type,
            op_type: &op_type,
            base_version,
            lamport,
            payload_json: &payload_json,
            conflict,
        })?;

        let mut vector = read_vector(tx, &ty, &id)?;
        vector.insert(actor_id.clone(), lamport);
        let vector_json = serde_json::to_string(&vector).unwrap_or_else(|_| "{}".to_string());

        // A conflicting op does not advance the converged state — resolution
        // decides whether the editor's change is applied on top (accept-client)
        // or discarded in favour of the newer server state (accept-server).
        let new_version = if conflict { current } else { lamport };
        upsert_state(tx, &ty, &id, new_version, &vector_json)?;

        Ok(serde_json::json!({
            "op_id": op_id,
            "lamport": lamport,
            "current_version": new_version,
            "conflict": conflict,
            "base_version": base_version,
        }))
    });

    let body = outcome.map_err(|e| {
        err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e)
    })?;

    Ok(Json(body))
}
/// `GET /api/collab/ops?resource_type=&resource_id=&since=&limit=` — one bounded
/// page of the op-log since a Lamport timestamp (exclusive).
///
/// Pagination semantics: rows are ordered by `lamport_ts ASC`, at most
/// `OPS_PAGE_LIMIT` are returned per call (`?limit=` may ask for fewer), and the
/// cursor for the next page is the `lamport` field of the last returned item —
/// pass it back as `since`. An empty (or short) page means the end of the log.
/// Both the row count and each op's `payload` are server-capped so no caller can
/// force an unbounded DB read, heap allocation or response.
pub async fn list_ops(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<OpsQuery>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(locale, StatusCode::BAD_REQUEST, "error-collab-invalid-resource"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if !has_collab_access(&mut conn, &user, &params.resource_type, &params.resource_id) {
        return Err(err(locale, StatusCode::FORBIDDEN, "error-collab-owner-only"));
    }

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        actor_id: String,
        #[diesel(sql_type = Text)]
        actor_name: String,
        #[diesel(sql_type = Text)]
        actor_type: String,
        #[diesel(sql_type = Text)]
        op_type: String,
        #[diesel(sql_type = BigInt)]
        base_version: i64,
        #[diesel(sql_type = BigInt)]
        lamport_ts: i64,
        #[diesel(sql_type = Text)]
        payload: String,
        #[diesel(sql_type = Bool)]
        conflict: bool,
        #[diesel(sql_type = Bool)]
        resolved: bool,
        #[diesel(sql_type = Text)]
        resolution: String,
        #[diesel(sql_type = Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let since = params.since.unwrap_or(0);
    let limit = clamp_ops_limit(params.limit);
    let rows = diesel::sql_query(LIST_OPS_SQL)
        .bind::<Text, _>(&params.resource_type)
        .bind::<Text, _>(&params.resource_id)
        .bind::<BigInt, _>(since)
        .bind::<BigInt, _>(limit)
        .load::<Row>(&mut conn)
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let payload = capped_payload(&r.payload);
            serde_json::json!({
                "id": r.id,
                "actor_id": r.actor_id,
                "actor_name": r.actor_name,
                "actor_type": r.actor_type,
                "op_type": r.op_type,
                "base_version": r.base_version,
                "lamport": r.lamport_ts,
                "payload": payload,
                "conflict": r.conflict,
                "resolved": r.resolved,
                "resolution": r.resolution,
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(items))
}

#[cfg(test)]
mod tests {
    use super::{
        capped_payload, clamp_ops_limit, LIST_OPS_SQL, OPS_PAGE_LIMIT, OPS_PAYLOAD_BYTE_CAP,
    };
    use crate::main_module::routes::collab_routes::grants_collab_access;

    /// Two-tenant fixture for the pure predicate shared by submit/list. The
    /// role is resolved by `effective_role` from verified claims; a foreign
    /// tenant has no grant on the id and therefore resolves to `None`.
    #[test]
    fn ops_access_denies_foreign_and_unknown_roles() {
        let grants = [
            (("tenant-a", "doc-a"), "owner"),
            (("tenant-a", "doc-shared"), "editor"),
            (("tenant-b", "doc-b"), "owner"),
        ];
        let role_for = |tenant: &str, doc: &str| {
            grants
                .iter()
                .find(|((t, d), _)| *t == tenant && *d == doc)
                .map(|(_, role)| *role)
        };
        assert!(grants_collab_access(role_for("tenant-a", "doc-a")));
        assert!(grants_collab_access(role_for("tenant-a", "doc-shared")));
        assert!(!grants_collab_access(role_for("tenant-a", "doc-b")));
        assert!(!grants_collab_access(role_for("tenant-b", "doc-a")));
        assert!(!grants_collab_access(None));
        assert!(!grants_collab_access(Some("administrator")));
    }

    /// Static guard: every ops handler must gate on `has_collab_access` inside
    /// its own body, so a handler copied without the guard fails this test.
    #[test]
    fn every_ops_handler_is_access_checked() {
        for (src, name) in [
            (include_str!("read_current_version.rs"), "submit_op"),
            (include_str!("read_current_version.rs"), "list_ops"),
            (include_str!("list_conflicts.rs"), "list_conflicts"),
            (include_str!("list_conflicts.rs"), "resolve_conflict"),
            (include_str!("list_conflicts.rs"), "list_actors"),
            (include_str!("list_conflicts.rs"), "ops_state"),
        ] {
            let marker = format!("pub async fn {name}(");
            assert!(src.contains(&marker), "{name} handler missing");
            let from = src.find(&marker).unwrap_or(0);
            let tail = &src[from..];
            let next = tail[marker.len()..]
                .find("pub async fn")
                .map(|offset| marker.len() + offset)
                .unwrap_or(tail.len());
            assert!(
                tail[..next].contains("has_collab_access("),
                "{name} is missing the access guard"
            );
        }
    }

    #[test]
    fn page_limit_is_always_bounded() {
        assert_eq!(clamp_ops_limit(None), OPS_PAGE_LIMIT);
        assert_eq!(clamp_ops_limit(Some(0)), 1);
        assert_eq!(clamp_ops_limit(Some(-7)), 1);
        assert_eq!(clamp_ops_limit(Some(1)), 1);
        assert_eq!(clamp_ops_limit(Some(250)), 250);
        assert_eq!(clamp_ops_limit(Some(OPS_PAGE_LIMIT)), OPS_PAGE_LIMIT);
        assert_eq!(clamp_ops_limit(Some(OPS_PAGE_LIMIT + 1)), OPS_PAGE_LIMIT);
        assert_eq!(clamp_ops_limit(Some(i64::MAX)), OPS_PAGE_LIMIT);
    }

    #[test]
    fn list_ops_query_always_carries_a_limit() {
        assert!(LIST_OPS_SQL.contains("LIMIT $4"), "{LIST_OPS_SQL}");
        assert!(LIST_OPS_SQL.contains("ORDER BY lamport_ts ASC"));
        assert!(LIST_OPS_SQL.contains("lamport_ts > $3"));
    }

    #[test]
    fn oversized_or_malformed_payloads_are_capped_to_null() {
        let small = r#"{"op":"insert","n":1}"#;
        assert_eq!(capped_payload(small)["op"], "insert");

        let huge = format!("{{\"blob\":\"{}\"}}", "x".repeat(OPS_PAYLOAD_BYTE_CAP + 64));
        assert_eq!(capped_payload(&huge), serde_json::Value::Null);
        assert_eq!(capped_payload("{not json"), serde_json::Value::Null);
    }
}
