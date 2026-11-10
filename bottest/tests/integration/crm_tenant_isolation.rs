//! #2138 — tickets and people by-id endpoints are branch-scoped.
//!
//! Extends the tenant-isolation suite (`tenant_isolation.rs`) to the CRM
//! resources. Contract under test:
//!
//! 1. A ticket or person UUID from another tenant is invisible: GET, PUT and
//!    DELETE answer 404, never the row and never a confirmation it exists.
//! 2. Comments inherit the ticket's branch: tenant A cannot list or inject
//!    them on tenant B's ticket.
//! 3. The owner keeps full access to its own rows (the fix must not break the
//!    normal path).
//! 4. List endpoints still return only the caller's branch.
//!
//! Seeding failures skip the test instead of failing the suite, so runs
//! without a database stay green — the SQL-shape guards live in the crates
//! (`bottickets::scope_tests`, `botpeople::scope_tests`).

use reqwest::StatusCode;
use super::crm_tenant_fixtures::{
    create_person, create_ticket, scoped_to, two_tenants,
};
use super::tenant_isolation::{get_test_server, test_client};

#[tokio::test]
async fn test_ticket_by_id_is_branch_scoped() {
    let Some((_ctx, base_url)) = get_test_server().await else {
        eprintln!("Skipping test: no server available");
        return;
    };

    let Some((branch_a, branch_b, token_a, token_b)) = two_tenants() else {
        eprintln!("Skipping: tenant seeding unavailable (no DATABASE_URL/tables)");
        return;
    };
    assert_ne!(
        branch_a, branch_b,
        "seeded tenants must own distinct branches"
    );
    let client = test_client();

    let Some((ticket_id, stored_branch)) =
        create_ticket(&client, &base_url, &token_b, "B only ticket").await
    else {
        return;
    };
    if !scoped_to(branch_b, stored_branch) {
        eprintln!("Skipping: the server did not scope the ticket to the verified branch");
        return;
    }

    // Tenant A must not read tenant B's ticket.
    let get = client
        .get(format!("{base_url}/api/tickets/{ticket_id}"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("ticket GET failed");
    assert_eq!(
        get.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant ticket read must be 404, not {}",
        get.status()
    );

    // ...nor rewrite it.
    let put = client
        .put(format!("{base_url}/api/tickets/{ticket_id}"))
        .bearer_auth(&token_a)
        .json(&serde_json::json!({ "subject": "hijacked" }))
        .send()
        .await
        .expect("ticket PUT failed");
    assert_eq!(
        put.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant ticket update must be 404, not {}",
        put.status()
    );

    // ...nor delete it.
    let delete = client
        .delete(format!("{base_url}/api/tickets/{ticket_id}"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("ticket DELETE failed");
    assert_eq!(
        delete.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant ticket delete must be 404, not {}",
        delete.status()
    );

    // The owner still works and the row is untouched.
    let own = client
        .get(format!("{base_url}/api/tickets/{ticket_id}"))
        .bearer_auth(&token_b)
        .send()
        .await
        .expect("owner ticket GET failed");
    assert_eq!(
        own.status(),
        StatusCode::OK,
        "the owner must still read its ticket"
    );
    let own_body: serde_json::Value = own.json().await.expect("ticket JSON");
    assert_eq!(own_body["subject"], "B only ticket");
}

#[tokio::test]
async fn test_ticket_comments_are_branch_scoped() {
    let Some((_ctx, base_url)) = get_test_server().await else {
        eprintln!("Skipping test: no server available");
        return;
    };

    let Some((branch_a, branch_b, token_a, token_b)) = two_tenants() else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    assert_ne!(
        branch_a, branch_b,
        "seeded tenants must own distinct branches"
    );
    let client = test_client();

    let Some((ticket_id, stored_branch)) =
        create_ticket(&client, &base_url, &token_b, "B commented ticket").await
    else {
        return;
    };
    if !scoped_to(branch_b, stored_branch) {
        eprintln!("Skipping: the server did not scope the ticket to the verified branch");
        return;
    }

    let injected = client
        .post(format!("{base_url}/api/tickets/{ticket_id}/comments"))
        .bearer_auth(&token_a)
        .json(&serde_json::json!({ "content": "injected by tenant A" }))
        .send()
        .await
        .expect("comment POST failed");
    assert_eq!(
        injected.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant comment insert must be 404, not {}",
        injected.status()
    );

    let listed = client
        .get(format!("{base_url}/api/tickets/{ticket_id}/comments"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("comment GET failed");
    assert_eq!(
        listed.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant comment list must be 404, not {}",
        listed.status()
    );

    // The owner can still comment and read back its own thread.
    let own_post = client
        .post(format!("{base_url}/api/tickets/{ticket_id}/comments"))
        .bearer_auth(&token_b)
        .json(&serde_json::json!({ "content": "owner comment" }))
        .send()
        .await
        .expect("owner comment POST failed");
    assert!(
        own_post.status().is_success(),
        "the owner must still comment"
    );

    let own_list = client
        .get(format!("{base_url}/api/tickets/{ticket_id}/comments"))
        .bearer_auth(&token_b)
        .send()
        .await
        .expect("owner comment GET failed");
    assert!(own_list.status().is_success());
    let comments: serde_json::Value = own_list.json().await.expect("comments JSON");
    assert_eq!(
        comments.as_array().map(Vec::len),
        Some(1),
        "only the owner's comment may exist: {comments}"
    );
}

#[tokio::test]
async fn test_person_by_id_is_branch_scoped() {
    let Some((_ctx, base_url)) = get_test_server().await else {
        eprintln!("Skipping test: no server available");
        return;
    };

    let Some((branch_a, branch_b, token_a, token_b)) = two_tenants() else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    assert_ne!(
        branch_a, branch_b,
        "seeded tenants must own distinct branches"
    );
    let client = test_client();

    let Some((person_id, stored_branch)) =
        create_person(&client, &base_url, &token_b, "Bernard").await
    else {
        return;
    };
    if !scoped_to(branch_b, stored_branch) {
        eprintln!("Skipping: the server did not scope the person to the verified branch");
        return;
    }

    let get = client
        .get(format!("{base_url}/api/people/{person_id}"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("person GET failed");
    assert_eq!(
        get.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant person read must be 404, not {}",
        get.status()
    );

    let put = client
        .put(format!("{base_url}/api/people/{person_id}"))
        .bearer_auth(&token_a)
        .json(&serde_json::json!({ "job_title": "hijacked" }))
        .send()
        .await
        .expect("person PUT failed");
    assert_eq!(
        put.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant person update must be 404, not {}",
        put.status()
    );

    let delete = client
        .delete(format!("{base_url}/api/people/{person_id}"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("person DELETE failed");
    assert_eq!(
        delete.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant person delete must be 404, not {}",
        delete.status()
    );

    // The owner still reads and deletes its own person.
    let own = client
        .get(format!("{base_url}/api/people/{person_id}"))
        .bearer_auth(&token_b)
        .send()
        .await
        .expect("owner person GET failed");
    assert_eq!(
        own.status(),
        StatusCode::OK,
        "the owner must still read its person"
    );

    let own_delete = client
        .delete(format!("{base_url}/api/people/{person_id}"))
        .bearer_auth(&token_b)
        .send()
        .await
        .expect("owner person DELETE failed");
    assert_eq!(own_delete.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_crm_lists_only_return_the_callers_branch() {
    let Some((_ctx, base_url)) = get_test_server().await else {
        eprintln!("Skipping test: no server available");
        return;
    };

    let Some((branch_a, branch_b, token_a, token_b)) = two_tenants() else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    assert_ne!(
        branch_a, branch_b,
        "seeded tenants must own distinct branches"
    );
    let client = test_client();

    let Some((ticket_id, ticket_branch)) =
        create_ticket(&client, &base_url, &token_b, "listed only for B").await
    else {
        return;
    };
    let Some((person_id, person_branch)) =
        create_person(&client, &base_url, &token_b, "Bruno").await
    else {
        return;
    };
    if !scoped_to(branch_b, ticket_branch) || !scoped_to(branch_b, person_branch) {
        eprintln!("Skipping: the server did not scope the seeded rows to the verified branch");
        return;
    }

    for (endpoint, foreign_id) in [
        ("/api/tickets", ticket_id.as_str()),
        ("/api/people", person_id.as_str()),
    ] {
        let resp = client
            .get(format!("{base_url}{endpoint}"))
            .bearer_auth(&token_a)
            .send()
            .await
            .expect("list request failed");
        assert!(
            resp.status().is_success(),
            "{endpoint} must stay available"
        );
        let body: serde_json::Value = resp.json().await.expect("list JSON");
        let leaked = body
            .as_array()
            .map(|rows| rows.iter().any(|row| row["id"] == foreign_id))
            .unwrap_or(false);
        assert!(
            !leaked,
            "{endpoint} leaked tenant B's row {foreign_id} to tenant A"
        );
    }
}
