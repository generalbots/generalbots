use super::*;

const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";
const USER_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const USER_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const STRANGER: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";
const DIM: usize = 4;
const QUERY: [f32; DIM] = [1.0, 0.0, 0.0, 0.0];

fn parse(value: &str) -> uuid::Uuid {
    match uuid::Uuid::parse_str(value) {
        Ok(id) => id,
        Err(_) => uuid::Uuid::nil(),
    }
}

fn user(tenant_id: &str, user_id: &str) -> UserContext {
    UserContext::authenticated(parse(user_id), None, Some(parse(tenant_id)))
}

/// Builds a point through the real projection: StructuredChunk ->
/// to_upsert_point -> filled vector, exactly as the runtime indexer does.
fn point(
    tenant_id: &str,
    document: &str,
    marker: &str,
    allowed_user: &str,
    embedding: [f32; DIM],
) -> UpsertPoint {
    use super::super::chunk::stable_id;
    use super::super::StructuredChunk;
    let chunk = StructuredChunk {
        id: stable_id(&[tenant_id, document, marker]),
        document_id: document.to_string(),
        file: format!("{document}-{tenant_id}.docx"),
        version: "3".to_string(),
        tenant_id: tenant_id.to_string(),
        heading_path: vec!["Employment".to_string(), "Termination".to_string()],
        source: SourceRef::clause("7.3").with_page(12),
        content: format!("CONFIDENTIAL_{marker}: the {marker} notice period is ninety days."),
        acl: AclMetadata::private(tenant_id).with_users(vec![allowed_user.to_string()]),
    };
    let mut upsert = chunk.to_upsert_point(DIM);
    upsert.vector = VectorSlot::filled(embedding.to_vec());
    upsert
}

#[test]
fn search_is_tenant_scoped_and_acl_filtered() {
    let mut index = InMemoryKbIndex::new();
    let alpha = point(TENANT_A, "contract", "alpha", USER_A, QUERY);
    let beta = point(TENANT_B, "contract", "beta", USER_B, QUERY);
    let id_alpha = alpha.id.clone();
    let id_beta = beta.id.clone();
    assert_eq!(index.insert(vec![alpha, beta]), Ok(2));
    assert_eq!(index.len(), 2);

    let hits_a = index.search(&QUERY, &user(TENANT_A, USER_A), 10);
    assert_eq!(
        hits_a.len(),
        1,
        "only tenant A's permitted point is visible"
    );
    assert_eq!(hits_a[0].id, id_alpha);
    assert_eq!(hits_a[0].source.locator, "7.3");
    assert_eq!(hits_a[0].section.as_deref(), Some("Termination"));
    assert!(hits_a[0].score > 0.99);

    let hits_b = index.search(&QUERY, &user(TENANT_B, USER_B), 10);
    assert_eq!(hits_b.len(), 1, "tenant B sees only its own point");
    assert_eq!(hits_b[0].id, id_beta);
    assert_ne!(hits_a[0].id, hits_b[0].id);

    // Identical embeddings, wrong tenant: no hits at all.
    assert!(index.search(&QUERY, &user(TENANT_A, USER_B), 10).is_empty());
    // Right tenant, but the user is not on the ACL: still no hits.
    assert!(index
        .search(&QUERY, &user(TENANT_A, STRANGER), 10)
        .is_empty());
    // Anonymous users never pass a private ACL.
    let anonymous = UserContext {
        organization_id: Some(parse(TENANT_A)),
        ..UserContext::anonymous()
    };
    assert!(index.search(&QUERY, &anonymous, 10).is_empty());
}

#[test]
fn search_ranks_by_cosine_and_honours_limit() {
    let mut index = InMemoryKbIndex::new();
    let close = point(TENANT_A, "close", "close", USER_A, [1.0, 0.1, 0.0, 0.0]);
    let far = point(TENANT_A, "far", "far", USER_A, [0.2, 1.0, 0.0, 0.0]);
    index.insert(vec![close, far]);

    let hits = index.search(&QUERY, &user(TENANT_A, USER_A), 1);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].source.locator, "7.3");
    assert!(hits[0].score > 0.9, "closest embedding must rank first");
    assert_eq!(
        index.search(&QUERY, &user(TENANT_A, USER_A), 0).len(),
        0,
        "zero limit returns nothing"
    );
    // Zero-norm and wrong-dimension queries are inert, never an error.
    assert!(index
        .search(&[0.0, 0.0, 0.0, 0.0], &user(TENANT_A, USER_A), 5)
        .is_empty());
    assert!(index
        .search(&[1.0, 0.0], &user(TENANT_A, USER_A), 5)
        .is_empty());
}

#[test]
fn remove_drops_matching_prefix_without_touching_other_tenants() {
    let mut index = InMemoryKbIndex::new();
    let alpha = point(TENANT_A, "alpha", "alpha", USER_A, QUERY);
    let beta = point(TENANT_B, "beta", "beta", USER_B, QUERY);
    let id_beta = beta.id.clone();
    index.insert(vec![alpha, beta]);

    let prefix = format!("alpha-{TENANT_A}");
    assert_eq!(index.remove(&prefix), 1);
    assert!(index.search(&QUERY, &user(TENANT_A, USER_A), 10).is_empty());
    assert_eq!(
        index.search(&QUERY, &user(TENANT_B, USER_B), 10).len(),
        1,
        "tenant B must survive an unrelated delete"
    );

    assert_eq!(
        index.remove(&id_beta[..8]),
        1,
        "stable id prefix also matches"
    );
    assert!(index.is_empty());
    assert_eq!(index.remove(&prefix), 0);
}

#[test]
fn insert_validates_points_and_is_atomic() {
    let valid = point(TENANT_A, "valid", "valid", USER_A, QUERY);

    let mut empty_id = valid.clone();
    empty_id.id = String::new();
    assert_eq!(
        InMemoryKbIndex::new().insert(vec![empty_id.clone()]),
        Err(IndexError::EmptyId)
    );

    let mut non_finite = valid.clone();
    non_finite.vector.values[0] = f32::NAN;
    assert!(matches!(
        InMemoryKbIndex::new().insert(vec![non_finite]),
        Err(IndexError::NonFiniteVector { .. })
    ));

    let mut bad_dimension = valid.clone();
    bad_dimension.vector = VectorSlot::filled(vec![1.0, 0.0]);
    // Dimension is established by the first insert; build an index that has
    // already accepted a 4-dim point so a 2-dim point is a real mismatch.
    let mut fixed = InMemoryKbIndex::new();
    assert!(fixed.insert(vec![valid.clone()]).is_ok(), "first insert should succeed");
    assert!(matches!(
        fixed.insert(vec![bad_dimension]),
        Err(IndexError::DimensionMismatch { .. })
    ));

    let mut missing_acl = valid.clone();
    missing_acl.payload.remove("acl");
    assert!(matches!(
        InMemoryKbIndex::new().insert(vec![missing_acl]),
        Err(IndexError::MissingPayloadField { field: "acl", .. })
    ));

    let mut mismatched_tenant = valid.clone();
    mismatched_tenant
        .payload
        .insert("tenant_id".to_string(), serde_json::json!(TENANT_B));
    assert!(matches!(
        InMemoryKbIndex::new().insert(vec![mismatched_tenant]),
        Err(IndexError::TenantMismatch { .. })
    ));

    let mut index = InMemoryKbIndex::new();
    assert!(index.insert(vec![valid.clone(), empty_id]).is_err());
    assert!(
        index.is_empty(),
        "a rejected batch must not be partially stored"
    );
    assert_eq!(index.insert(Vec::new()), Ok(0));
    assert_eq!(index.insert(vec![valid]), Ok(1));
}

#[test]
fn results_and_index_never_expose_document_content() {
    let mut index = InMemoryKbIndex::new();
    assert_eq!(
        index.insert(vec![point(TENANT_A, "secret", "TOPSECRET", USER_A, QUERY)]),
        Ok(1)
    );
    let hits = index.search(&QUERY, &user(TENANT_A, USER_A), 10);
    assert_eq!(hits.len(), 1);
    let debug = format!("{hits:?} {index:?}");
    assert!(
        !debug.contains("TOPSECRET") && !debug.contains("CONFIDENTIAL"),
        "document text leaked through Debug"
    );
}
