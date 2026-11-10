//! Contract tests for the SYNCHRONIZE record transform (#2064).
//!
//! `transform_record` is the pure half of the ETL pipeline: field mapping,
//! inclusion/exclusion and type coercion happen here, before any SQL. The
//! service is built directly because these methods never touch the pool or the
//! HTTP client.

use super::*;

fn service() -> SynchronizeService {
    SynchronizeService {
        http_client: reqwest::Client::new(),
        base_url: None,
    }
}

fn config(transform: Option<TransformConfig>) -> SynchronizeConfig {
    SynchronizeConfig {
        endpoint: "/api/customers".to_string(),
        table_name: "customers".to_string(),
        key_field: "id".to_string(),
        page_param: "page".to_string(),
        limit_param: "limit".to_string(),
        page_size: None,
        headers: None,
        transform,
        conflict_resolution: None,
        batch_size: None,
        dry_run: None,
    }
}

fn transform(
    mappings: Option<(&str, &str)>,
    exclude: Option<&[&str]>,
    include: Option<&[&str]>,
) -> TransformConfig {
    TransformConfig {
        field_mappings: mappings.map(|(from, to)| {
            HashMap::from([(from.to_string(), to.to_string())])
        }),
        exclude_fields: exclude.map(|fields| fields.iter().map(|f| f.to_string()).collect()),
        include_fields: include.map(|fields| fields.iter().map(|f| f.to_string()).collect()),
        computed_fields: None,
        type_coercions: None,
    }
}

#[test]
fn a_record_without_transform_is_returned_untouched() {
    let record = serde_json::json!({"id": 1, "name": "Ana"});
    let result = service()
        .transform_record(&record, &config(None))
        .expect("no transform is not an error");
    assert_eq!(result, record);
}

#[test]
fn field_mappings_rename_the_target_key() {
    let record = serde_json::json!({"customer_name": "Ana", "id": 7});
    let config = config(Some(transform(Some(("customer_name", "name")), None, None)));

    let result = service().transform_record(&record, &config).expect("transform");
    assert_eq!(result, serde_json::json!({"name": "Ana", "id": 7}));
}

#[test]
fn exclude_and_include_filter_the_payload() {
    let record = serde_json::json!({"id": 7, "name": "Ana", "secret": "s3cr3t"});

    let excluded = config(Some(transform(None, Some(&["secret"]), None)));
    assert_eq!(
        service().transform_record(&record, &excluded).expect("transform"),
        serde_json::json!({"id": 7, "name": "Ana"})
    );

    let included = config(Some(transform(None, None, Some(&["id"]))));
    assert_eq!(
        service().transform_record(&record, &included).expect("transform"),
        serde_json::json!({"id": 7})
    );
}

#[test]
fn exclusion_wins_over_inclusion() {
    let record = serde_json::json!({"id": 7, "name": "Ana"});
    let config = config(Some(transform(None, Some(&["id"]), Some(&["id"]))));

    let result = service().transform_record(&record, &config).expect("transform");
    assert_eq!(result, serde_json::json!({}));
}

#[test]
fn coercions_apply_to_the_mapped_key() {
    let record = serde_json::json!({"amount": "12.50", "count": "3"});
    let mut settings = transform(Some(("count", "quantity")), None, None);
    settings.type_coercions = Some(HashMap::from([
        ("quantity".to_string(), FieldType::Integer),
        ("amount".to_string(), FieldType::Float),
    ]));

    let result = service()
        .transform_record(&record, &config(Some(settings)))
        .expect("transform");
    assert_eq!(result, serde_json::json!({"amount": 12.5, "quantity": 3}));
}

#[test]
fn a_failed_coercion_fails_the_record() {
    let record = serde_json::json!({"count": "many"});
    let mut settings = transform(None, None, None);
    settings.type_coercions =
        Some(HashMap::from([("count".to_string(), FieldType::Integer)]));

    let error = service()
        .transform_record(&record, &config(Some(settings)))
        .expect_err("a non-numeric count must not be stored");
    assert!(matches!(error, SynchronizeError::TransformFailed(_)));
}

#[test]
fn a_non_object_record_is_passed_through() {
    let record = serde_json::json!([1, 2, 3]);
    let result = service()
        .transform_record(&record, &config(Some(transform(None, None, None))))
        .expect("transform");
    assert_eq!(result, record);
}

#[test]
fn computed_fields_are_always_present() {
    let record = serde_json::json!({"id": 1});
    let mut settings = transform(None, None, None);
    settings.computed_fields = Some(vec![
        ComputedField {
            name: "created_at".to_string(),
            expression: "NOW()".to_string(),
            field_type: FieldType::DateTime,
        },
        ComputedField {
            name: "external_id".to_string(),
            expression: "UUID()".to_string(),
            field_type: FieldType::Uuid,
        },
        ComputedField {
            name: "label".to_string(),
            expression: "CONCAT(id, name)".to_string(),
            field_type: FieldType::String,
        },
        ComputedField {
            name: "unknown".to_string(),
            expression: "LOWER(name)".to_string(),
            field_type: FieldType::String,
        },
    ]);

    let result = service()
        .transform_record(&record, &config(Some(settings)))
        .expect("transform");

    let created_at = result["created_at"].as_str().expect("NOW() is a string");
    assert!(created_at.contains('T'), "{created_at} is not RFC 3339");
    let external_id = result["external_id"].as_str().expect("UUID() is a string");
    assert!(Uuid::parse_str(external_id).is_ok());
    // Unsupported expressions degrade to NULL instead of failing the page.
    assert!(result["unknown"].is_null());
}

#[test]
fn the_now_and_uuid_expressions_are_case_insensitive() {
    let field = |expression: &str| ComputedField {
        name: "v".to_string(),
        expression: expression.to_string(),
        field_type: FieldType::String,
    };
    let service = service();
    let record = serde_json::json!({});

    let now = service.compute_field(&record, &field("now()")).expect("now");
    assert!(now.as_str().is_some_and(|value| value.contains('T')));
    let uuid = service.compute_field(&record, &field("uuid()")).expect("uuid");
    assert!(Uuid::parse_str(uuid.as_str().unwrap_or_default()).is_ok());
}

/// The stub loaders never reach a database, so a record is always "new" and
/// every write reports success: the pipeline can be exercised without Postgres.
#[tokio::test]
async fn the_load_stubs_report_no_existing_record() {
    let service = service();
    let existing = service
        .find_existing_record(Uuid::nil(), "customers", "id", "1")
        .await
        .expect("lookup");
    assert!(existing.is_none());

    service
        .insert_record(Uuid::nil(), "customers", &serde_json::json!({"id": 1}))
        .await
        .expect("insert");
    service
        .update_record(
            Uuid::nil(),
            "customers",
            "id",
            "1",
            &serde_json::json!({"id": 1}),
        )
        .await
        .expect("update");
}

#[tokio::test]
async fn a_failed_page_is_retried_after_the_documented_delay() {
    let delay = service().handle_retry(1).await.expect("retry is configured");
    assert_eq!(delay, std::time::Duration::from_millis(1000));
}
