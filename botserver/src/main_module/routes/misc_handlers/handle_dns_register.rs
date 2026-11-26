use super::*;
use crate::security::auth_api::types::AuthenticatedUser;
use axum::response::Response;
use axum::Extension;

/// POST /api/dns/register
///
/// Inserts a DNS record only after (a) every field passes strict validation
/// and (b) the authenticated caller is shown to control the record's zone.
pub async fn handle_dns_register(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Form(form): Form<DnsRecordRequest>,
) -> Response {
    use diesel::prelude::*;

    let record = match validate_dns_record(
        form.hostname.as_deref(),
        form.record_type.as_deref(),
        form.target.as_deref(),
        form.ttl.as_deref(),
    ) {
        Ok(record) => record,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Html(format!(
                    "<div class=\"dns-error\">{}</div>",
                    html_escape(&e)
                )),
            )
                .into_response()
        }
    };

    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<div class=\"dns-error\">{}</div>",
                    html_escape(&e.1)
                )),
            )
                .into_response()
        }
    };

    let owns = match caller_owns_hostname(&mut conn, &user, &record.hostname) {
        Ok(owns) => owns,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<div class=\"dns-error\">{}</div>",
                    html_escape(&e)
                )),
            )
                .into_response()
        }
    };
    if !owns {
        return (
            StatusCode::FORBIDDEN,
            Html("<div class=\"dns-error\">You do not control this DNS zone</div>".to_string()),
        )
            .into_response();
    }

    let id = Uuid::new_v4();
    if let Err(e) = diesel::sql_query(
        "INSERT INTO dns_records (id, hostname, record_type, target, ttl, status) VALUES ($1, $2, $3, $4, $5, 'active') ON CONFLICT DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Text, _>(&record.hostname)
    .bind::<diesel::sql_types::Text, _>(&record.record_type)
    .bind::<diesel::sql_types::Text, _>(&record.target)
    .bind::<diesel::sql_types::Int4, _>(record.ttl)
    .execute(&mut conn)
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<div class=\"dns-error\">{}</div>",
                html_escape(&e.to_string())
            )),
        )
            .into_response();
    }

    Html(format!(
        r#"<div class="dns-success">DNS record <strong>{hostname}</strong> registered</div>"#,
        hostname = html_escape(&record.hostname),
    ))
    .into_response()
}

/// GET /api/dns/:id/edit
///
/// Only the record's owner may see its fragment. A foreign or unknown id is
/// indistinguishable from a missing record: both return 404.
pub async fn handle_dns_edit_form(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Extension(user): Extension<AuthenticatedUser>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Response {
    use diesel::prelude::*;

    // #2793: ownership is derived from the signed branch claim and the
    // authenticated principal, never from the requested record id.
    let branch_id = botsecurity_core::scope::branch_from_jwt_pool(&headers, &state.conn);
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<p>{}</p>", html_escape(&e.1))),
            )
                .into_response()
        }
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct DnsRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        hostname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        record_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        target: String,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        ttl: i32,
    }

    let row: Option<DnsRow> = match diesel::sql_query(
        "SELECT hostname, record_type, target, ttl FROM dns_records WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(&mut conn)
    {
        Ok(v) => Some(v),
        Err(e) => {
            log::debug!("DNS record lookup failed: {e}");
            None
        }
    };

    let not_found = || {
        (
            StatusCode::NOT_FOUND,
            Html("<p class=\"text-muted\">Record not found</p>".to_string()),
        )
            .into_response()
    };

    match row {
        Some(r) => {
            let owns = match caller_owns_hostname_in(&mut conn, &user, branch_id, &r.hostname) {
                Ok(owns) => owns,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Html(format!("<p>{}</p>", html_escape(&e))),
                    )
                        .into_response()
                }
            };
            if !owns {
                return not_found();
            }
            Html(format!(
                r##"<form hx-post="/api/dns/register" hx-target="#edit-dns-form-container" hx-swap="innerHTML">
            <div class="form-group"><label>Hostname</label><input type="text" name="hostname" value="{hostname}" required /></div>
            <div class="form-group"><label>Type</label><input type="text" name="record_type" value="{rtype}" /></div>
            <div class="form-group"><label>Target/IP</label><input type="text" name="target" value="{target}" /></div>
            <div class="form-group"><label>TTL</label><input type="number" name="ttl" value="{ttl}" /></div>
            <button type="submit" class="btn-primary btn-sm">Save</button>
        </form>"##,
                hostname = html_escape(&r.hostname),
                rtype = html_escape(&r.record_type),
                target = html_escape(&r.target),
                ttl = r.ttl,
            ))
            .into_response()
        }
        None => not_found(),
    }
}
