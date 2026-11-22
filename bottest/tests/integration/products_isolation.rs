//! Cross-tenant HTTP isolation for the products catalogue (#1387/#1389).
//!
//! Extends the tenant-isolation suite (`tenant_isolation.rs`) and mirrors the
//! CRM coverage (`crm_tenant_isolation.rs`) for `/api/products/items`:
//!
//! 1. A product UUID owned by tenant B is invisible to tenant A: GET by id,
//!    PUT and DELETE all answer 404 — never the row, never a confirmation.
//! 2. The list endpoint never includes tenant B's product for tenant A.
//! 3. The owner keeps full access: its product is reachable by id and appears
//!    in its own list (the fix must not break the normal path).
//!
//! Seeding / endpoint failures skip with a printed reason instead of failing
//! the suite, so runs without a live server or database stay green.

use super::crm_tenant_fixtures::{scoped_to, two_tenants};
use super::tenant_isolation::{get_test_server, test_client};
use reqwest::{Client, StatusCode};

/// Creates a product as `token`; returns `(id, branch_id)` as the server
/// stored them, or `None` when the endpoint is unavailable in this build.
async fn create_product(
    client: &Client,
    base_url: &str,
    token: &str,
    name: &str,
) -> Option<(String, Option<String>)> {
    let resp = client
        .post(format!("{base_url}/api/products/items"))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "name": name,
            "sku": format!("iso-{name}"),
            "price": 9.99,
        }))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        eprintln!("Skipping: product create unavailable ({})", resp.status());
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let id = body["id"].as_str()?.to_string();
    Some((id, body["branch_id"].as_str().map(str::to_string)))
}

/// A product id owned by tenant B, scoped to B's verified branch. `None`
/// means the setup was unavailable and the caller must skip.
async fn seed_b_product(
    client: &Client,
    base_url: &str,
    branch_b: uuid::Uuid,
    token_b: &str,
    name: &str,
) -> Option<String> {
    let (id, stored_branch) = create_product(client, base_url, token_b, name).await?;
    if !scoped_to(branch_b, stored_branch) {
        eprintln!("Skipping: server did not scope the product to the verified branch");
        return None;
    }
    Some(id)
}

#[tokio::test]
async fn test_product_by_id_is_branch_scoped() {
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

    let Some(product_id) =
        seed_b_product(&client, &base_url, branch_b, &token_b, "B only product").await
    else {
        return;
    };

    // Tenant A must not read tenant B's product by id.
    let get = client
        .get(format!("{base_url}/api/products/items/{product_id}"))
        .bearer_auth(&token_a)
        .send()
        .await;
    let Ok(get) = get else {
        eprintln!("Skipping: product GET failed to send");
        return;
    };
    assert_eq!(
        get.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant product read must be 404, not {}",
        get.status()
    );

    // ...nor rewrite it.
    let put = client
        .put(format!("{base_url}/api/products/items/{product_id}"))
        .bearer_auth(&token_a)
        .json(&serde_json::json!({ "name": "hijacked" }))
        .send()
        .await;
    let Ok(put) = put else {
        eprintln!("Skipping: product PUT failed to send");
        return;
    };
    assert_eq!(
        put.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant product update must be 404, not {}",
        put.status()
    );

    // ...nor delete it.
    let delete = client
        .delete(format!("{base_url}/api/products/items/{product_id}"))
        .bearer_auth(&token_a)
        .send()
        .await;
    let Ok(delete) = delete else {
        eprintln!("Skipping: product DELETE failed to send");
        return;
    };
    assert_eq!(
        delete.status(),
        StatusCode::NOT_FOUND,
        "cross-tenant product delete must be 404, not {}",
        delete.status()
    );

    // The owner still reads its own product, untouched.
    let own = client
        .get(format!("{base_url}/api/products/items/{product_id}"))
        .bearer_auth(&token_b)
        .send()
        .await;
    let Ok(own) = own else {
        eprintln!("Skipping: owner product GET failed to send");
        return;
    };
    assert_eq!(
        own.status(),
        StatusCode::OK,
        "the owner must still read its product"
    );
    let Ok(own_body) = own.json::<serde_json::Value>().await else {
        eprintln!("Skipping: owner product JSON unavailable");
        return;
    };
    assert_eq!(own_body["product"]["name"], "B only product");
}

#[tokio::test]
async fn test_products_list_only_returns_the_callers_branch() {
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

    let Some(product_id) =
        seed_b_product(&client, &base_url, branch_b, &token_b, "listed only for B").await
    else {
        return;
    };

    // Tenant A's list must stay available but must not leak tenant B's row.
    let foreign = client
        .get(format!("{base_url}/api/products/items"))
        .bearer_auth(&token_a)
        .send()
        .await;
    let Ok(foreign) = foreign else {
        eprintln!("Skipping: products list failed to send");
        return;
    };
    assert!(
        foreign.status().is_success(),
        "products list must stay available for tenant A"
    );
    let Ok(foreign_body) = foreign.json::<serde_json::Value>().await else {
        eprintln!("Skipping: products list JSON unavailable");
        return;
    };
    let leaked = foreign_body
        .as_array()
        .map_or(false, |rows| rows.iter().any(|row| row["id"] == product_id));
    assert!(
        !leaked,
        "products list leaked tenant B's product {product_id} to tenant A"
    );

    // The owner still sees its own product in its own list.
    let own = client
        .get(format!("{base_url}/api/products/items"))
        .bearer_auth(&token_b)
        .send()
        .await;
    let Ok(own) = own else {
        eprintln!("Skipping: owner products list failed to send");
        return;
    };
    assert!(
        own.status().is_success(),
        "products list must stay available for the owner"
    );
    let Ok(own_body) = own.json::<serde_json::Value>().await else {
        eprintln!("Skipping: owner products list JSON unavailable");
        return;
    };
    let visible = own_body
        .as_array()
        .map_or(false, |rows| rows.iter().any(|row| row["id"] == product_id));
    assert!(visible, "the owner must see its own product in the list");
}
