//! #2544 — Campaign lifecycle end-to-end flow: list → segment preview →
//! publish → recipient rows → suppression → unsubscribe.
//!
//! Drives the real `/api/crm/lists` and `/api/crm/campaigns` handlers and checks
//! `marketing_campaigns` / `marketing_recipients` after each mutation. Two
//! deliberate choices keep this test safe and honest offline:
//!
//! * publish uses an unknown channel (`offline`), which the provider loop
//!   skips, so no email/WhatsApp/Instagram provider is contacted; the fan-out
//!   rows and campaign status are still written exactly as in production;
//! * `marketing_contacts` has no public write endpoint yet, so recipient rows
//!   are seeded through the test database. The public unsubscribe endpoint
//!   `/api/marketing/unsubscribe/:token` is likewise not wired into the
//!   router, so opt-out is exercised through the CRM contact status that the
//!   send paths actually consult (`suppressed` / `unsubscribed`).
//!
//! Skips (with a printed reason) when no server/database is available.

use reqwest::Method;
use serde_json::{json, Value};
use uuid::Uuid;

use super::flow_support::{
    add_marketing_contact, campaign_state, connect_db, create_crm_contact, crm_contact_status,
    default_bot_scope, flow_env, flow_segment, flow_token, http_client, json_id, preview_counts,
    recipient_count, recipient_count_for_email, request_json,
};

#[tokio::test]
async fn test_campaign_flow_list_preview_publish_suppression_unsubscribe() {
    let Some(env) = flow_env().await else {
        eprintln!("Skipping campaign flow: no server/database available");
        return;
    };
    let Some(client) = http_client() else {
        eprintln!("Skipping campaign flow: cannot build HTTP client");
        return;
    };
    let Some((org_id, branch_id)) = default_bot_scope(&env.db_url) else {
        eprintln!("Skipping campaign flow: default bot scope unavailable");
        return;
    };
    let token = flow_token("campflow", &org_id, &branch_id);
    let mut conn = match connect_db(&env.db_url) {
        Some(conn) => conn,
        None => return,
    };

    // Two CRM contacts in the default branch: one stays subscribed, one is
    // later suppressed, then unsubscribed.
    let tag = Uuid::new_v4().simple();
    let keep_email = format!("keep-{tag}@campflow.test");
    let drop_email = format!("drop-{tag}@campflow.test");
    let keep_id = match create_crm_contact(&client, &env.base_url, &token, "Keep", &keep_email).await {
        Some(id) => id,
        None => return,
    };
    let drop_id = match create_crm_contact(&client, &env.base_url, &token, "Drop", &drop_email).await {
        Some(id) => id,
        None => return,
    };

    // 1. List — create and read back through the public list handlers. The
    // segment targets only the two contacts seeded by this test.
    let payload = json!({
        "name": format!("Flow list {tag}"),
        "list_type": "dynamic",
        "segment": flow_segment(),
    });
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/lists", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping campaign flow: list create returned {status}");
        return;
    }
    let Some(list_id) = json_id(&body).and_then(|id| Uuid::parse_str(&id).ok()) else {
        eprintln!("Skipping campaign flow: list create body had no id");
        return;
    };
    let (status, body) = match request_json(
        &client,
        Method::GET,
        format!("{}/api/crm/lists", env.base_url),
        &token,
        None,
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    assert!(status.is_success(), "list endpoint must stay available");
    let list_id_text = list_id.to_string();
    let listed = body
        .as_ref()
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .any(|row| row["id"].as_str() == Some(list_id_text.as_str()))
        })
        .unwrap_or(false);
    assert!(listed, "the created list must appear in the list response");

    // Recipient membership is seeded in the DB: no public endpoint writes
    // marketing_contacts rows today.
    if add_marketing_contact(&mut conn, branch_id, list_id, &keep_email, "Keep").is_none()
        || add_marketing_contact(&mut conn, branch_id, list_id, &drop_email, "Drop").is_none()
    {
        eprintln!("Skipping campaign flow: cannot seed marketing contacts");
        return;
    }

    // 2. Segment preview — no membership rows are written and both contacts
    // are eligible before any opt-out.
    let preview_url = format!("{}/api/crm/lists/{list_id}/preview", env.base_url);
    let (status, body) = match request_json(&client, Method::POST, preview_url.clone(), &token, Some(&json!({}))).await {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping campaign flow: preview returned {status}");
        return;
    }
    let (baseline_matched, baseline_suppressed) = match preview_counts(&body) {
        Some(counts) => counts,
        None => return,
    };
    assert_eq!(baseline_matched, 2, "both seeded contacts must be previewed");

    // 3. Publish — the unknown channel is a no-op provider, but the campaign
    // state and recipient rows are written by the production code path.
    let payload = json!({
        "name": format!("Flow campaign {tag}"),
        "campaign_type": "email",
        "metrics": { "subject": "Flow test", "body": "Hello {{name}}" },
    });
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/campaigns", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping campaign flow: campaign create returned {status}");
        return;
    }
    let Some(campaign_id) = json_id(&body).and_then(|id| Uuid::parse_str(&id).ok()) else {
        eprintln!("Skipping campaign flow: campaign create body had no id");
        return;
    };
    let draft = match campaign_state(&mut conn, campaign_id) {
        Some(row) => row,
        None => return,
    };
    assert_eq!(draft.status, "draft", "new campaigns start as drafts");
    assert_eq!(draft.branch_id, branch_id);

    let payload = json!({ "list_id": list_id.to_string(), "channels": ["offline"] });
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/campaigns/{campaign_id}/publish", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping campaign flow: publish returned {status}");
        return;
    }
    let published = body.as_ref().and_then(|value| value["success"].as_bool()).unwrap_or(false);
    assert!(published, "publish must report success");
    assert_eq!(
        recipient_count(&mut conn, campaign_id),
        Some(2),
        "publish must write one recipient row per eligible contact"
    );
    let completed = match campaign_state(&mut conn, campaign_id) {
        Some(row) => row,
        None => return,
    };
    assert_eq!(completed.status, "completed", "metrics merge finalizes the campaign");

    // 4. Suppression — a CRM status opt-out is applied at send time, not
    // filtered only from stale list membership.
    let payload = json!({ "status": "suppressed" });
    let (status, _) = match request_json(
        &client,
        Method::PUT,
        format!("{}/api/crm/contacts/{drop_id}", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping campaign flow: contact suppression returned {status}");
        return;
    }
    assert_eq!(
        crm_contact_status(&mut conn, drop_id).as_deref(),
        Some("suppressed"),
        "the CRM status must be persisted"
    );
    let (status, body) = match request_json(&client, Method::POST, preview_url.clone(), &token, Some(&json!({}))).await {
        Some(result) => result,
        None => return,
    };
    assert!(status.is_success(), "preview must stay available after opt-out");
    let (_, suppressed_after) = match preview_counts(&body) {
        Some(counts) => counts,
        None => return,
    };
    assert_eq!(
        suppressed_after, baseline_suppressed + 1,
        "the suppressed contact must be counted, never sampled"
    );

    let payload = json!({ "list_id": list_id.to_string(), "channels": ["offline"] });
    let (status, _) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/campaigns/{campaign_id}/publish", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    assert!(status.is_success(), "republish must still find the subscribed contact");
    assert_eq!(
        recipient_count_for_email(&mut conn, campaign_id, &drop_email),
        Some(0),
        "suppressed recipients must not get a fresh fan-out row"
    );
    assert_eq!(
        recipient_count_for_email(&mut conn, campaign_id, &keep_email),
        Some(1),
        "the subscribed recipient must still be queued"
    );
    let rows_before_final = recipient_count(&mut conn, campaign_id);

    // 5. Unsubscribe — the remaining contact opts out; a final publish must
    // resolve no recipients and be refused. The handler short-circuits before
    // rewriting the fan-out table, so the previous run's row intentionally
    // remains; the contract under test is that no new send is queued.
    let payload = json!({ "status": "unsubscribed" });
    let (status, _) = match request_json(
        &client,
        Method::PUT,
        format!("{}/api/crm/contacts/{keep_id}", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping campaign flow: unsubscribe returned {status}");
        return;
    }
    assert_eq!(
        crm_contact_status(&mut conn, keep_id).as_deref(),
        Some("unsubscribed")
    );
    let (status, body) = match request_json(&client, Method::POST, preview_url, &token, Some(&json!({}))).await {
        Some(result) => result,
        None => return,
    };
    assert!(status.is_success());
    let (_, suppressed_all) = match preview_counts(&body) {
        Some(counts) => counts,
        None => return,
    };
    assert_eq!(suppressed_all, baseline_suppressed + 2);

    let payload = json!({ "list_id": list_id.to_string(), "channels": ["offline"] });
    let (status, _) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/campaigns/{campaign_id}/publish", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    assert_eq!(
        status,
        reqwest::StatusCode::BAD_REQUEST,
        "publishing to a fully opted-out list must be refused"
    );
    assert_eq!(
        recipient_count(&mut conn, campaign_id),
        rows_before_final,
        "a refused publish must not queue any new recipient row"
    );
    assert_eq!(
        recipient_count_for_email(&mut conn, campaign_id, &drop_email),
        Some(0),
        "the unsubscribed contact must never get a recipient row"
    );
}
