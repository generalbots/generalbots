    use super::*;
    use botlib::traits::{
        BoxFutureBool, BoxFutureDriveList, BoxFutureOptionDriveMeta, BoxFutureUnit,
        BoxFutureVecDriveObject, BoxFutureVecString, BoxFutureVecU8, DriveListResult,
    };
    use uuid::Uuid;

    const TENANT: &str = "11111111-1111-1111-1111-111111111111";
    const OWNER: &str = "33333333-3333-3333-3333-333333333333";
    const FOREIGN_ORG: &str = "22222222-2222-2222-2222-222222222222";
    const FOREIGN_USER: &str = "55555555-5555-5555-5555-555555555555";
    const HTML: &str = "<h1>Employment</h1><h2>Termination</h2>\
                         <p>7.3 Notice</p>\
                         <p>The notice period is ninety days. ZANZIBARCLAUSE.</p>";

    #[derive(Debug)]
    struct TestDrive;

    macro_rules! noop_drive_methods {
        ($($method:ident($($arg:ident : $ty:ty),*) -> $ret:ty => $value:expr;)*) => {
            impl DriveRepository for TestDrive {
                $(
                    fn $method(&self, $($arg: $ty),*) -> $ret {
                        let _ = ($($arg,)*);
                        Box::pin(async { Ok($value) })
                    }
                )*
            }
        };
    }
    noop_drive_methods! {
        put_object(bucket: &str, key: &str, data: Vec<u8>, content_type: Option<&str>) -> BoxFutureUnit => ();
        get_object(bucket: &str, key: &str) -> BoxFutureVecU8 => Vec::new();
        delete_object(bucket: &str, key: &str) -> BoxFutureUnit => ();
        copy_object(bucket: &str, from_key: &str, to_key: &str) -> BoxFutureUnit => ();
        list_objects(bucket: &str, prefix: Option<&str>) -> BoxFutureVecString => Vec::new();
        list_objects_with_metadata(bucket: &str, prefix: Option<&str>) -> BoxFutureVecDriveObject => Vec::new();
        list_common_prefixes(bucket: &str, delimiter: &str) -> BoxFutureVecString => Vec::new();
        list_all_buckets() -> BoxFutureVecString => Vec::new();
        object_exists(bucket: &str, key: &str) -> BoxFutureBool => false;
        get_object_metadata(bucket: &str, key: &str) -> BoxFutureOptionDriveMeta => None;
        create_bucket_if_not_exists(bucket: &str) -> BoxFutureUnit => ();
        delete_objects(bucket: &str, keys: Vec<String>) -> BoxFutureUnit => ();
        head_bucket(bucket: &str) -> BoxFutureBool => false;
        list_objects_v2(bucket: &str, prefix: &str, delimiter: Option<&str>) -> BoxFutureDriveList => DriveListResult { objects: Vec::new(), common_prefixes: Vec::new() };
        upload_file(bucket: &str, key: &str, file_path: &str, content_type: Option<&str>) -> BoxFutureUnit => ();
        download_file(bucket: &str, key: &str, file_path: &str) -> BoxFutureUnit => ();
    }

    fn uuid(value: &str) -> Uuid {
        Uuid::parse_str(value).unwrap_or_default()
    }

    fn state_with_bucket(bucket: &str) -> Arc<DocState> {
        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
            "postgres://unused:unused@127.0.0.1:1/unused",
        );
        let pool = diesel::r2d2::Pool::builder()
            .max_size(1)
            .build_unchecked(manager);
        Arc::new(DocState::with_in_memory_kb_index(
            Arc::new(pool),
            Arc::new(TestDrive),
            bucket.to_string(),
            Arc::new(MetricsCollector::default()),
        ))
    }

    #[test]
    fn saved_document_is_indexed_and_cited_for_owner() {
        let state = state_with_bucket(TENANT);
        state.project_to_kb(OWNER, "doc-1", "Employment", HTML);

        let owner = UserContext::authenticated(uuid(OWNER), None, Some(uuid(TENANT)));
        let hits = state.search_text("ZANZIBARCLAUSE notice", &owner, 5);
        assert_eq!(hits.len(), 1, "saved clause must be searchable");
        let hit = &hits[0];
        assert!(hit.score > 0.0);
        assert_eq!(hit.source.locator, "7.3");
        assert!(!hit.id.is_empty());
        let debug = format!("{hit:?}");
        assert!(
            !debug.contains("ninety days"),
            "citation-only hit leaked content: {debug}"
        );
    }

    #[test]
    fn deleted_document_is_removed_from_index() {
        let state = state_with_bucket(TENANT);
        let owner = UserContext::authenticated(uuid(OWNER), None, Some(uuid(TENANT)));
        state.project_to_kb(OWNER, "doc-del", "Employment", HTML);
        assert_eq!(state.search_text("ZANZIBARCLAUSE", &owner, 5).len(), 1);
        state.remove_from_kb("doc-del");
        assert!(state.search_text("ZANZIBARCLAUSE", &owner, 5).is_empty());
    }

    #[test]
    fn named_bucket_is_searchable_via_deterministic_tenant_uuid() {
        let state = state_with_bucket("default.gbai");
        state.project_to_kb(OWNER, "doc-named", "Employment", HTML);
        let tenant = DocState::tenant_uuid("default.gbai");
        let owner = UserContext::authenticated(uuid(OWNER), None, Some(tenant));
        let hits = state.search_text("ZANZIBARCLAUSE notice", &owner, 5);
        assert_eq!(hits.len(), 1, "named bucket must map to the projected tenant");
        assert_eq!(hits[0].source.locator, "7.3");
    }

    #[test]
    fn foreign_org_and_denied_user_get_no_hits() {
        let state = state_with_bucket(TENANT);
        state.project_to_kb(OWNER, "doc-2", "Employment", HTML);

        let foreign = UserContext::authenticated(uuid(OWNER), None, Some(uuid(FOREIGN_ORG)));
        let denied = UserContext::authenticated(uuid(FOREIGN_USER), None, Some(uuid(TENANT)));
        let query = lexical_embedding("ZANZIBARCLAUSE notice", DEFAULT_VECTOR_DIMENSION);
        assert!(state.search_index(&query, &foreign, 5).is_empty());
        assert!(state.search_index(&query, &denied, 5).is_empty());
    }

    #[test]
    fn editing_document_replaces_indexed_points() {
        let state = state_with_bucket(TENANT);
        let owner = UserContext::authenticated(uuid(OWNER), None, Some(uuid(TENANT)));
        state.project_to_kb(OWNER, "doc-edit", "Employment", HTML);
        assert_eq!(state.search_text("ZANZIBARCLAUSE", &owner, 5).len(), 1);

        let revised = "<h1>Employment</h1><h2>Termination</h2><p>7.3 Notice</p>\
                       <p>The notice period is sixty days. QUASARCLAUSE.</p>";
        state.project_to_kb(OWNER, "doc-edit", "Employment", revised);
        assert!(
            state.search_text("ZANZIBARCLAUSE", &owner, 5).is_empty(),
            "stale chunks must be evicted when a document is edited"
        );
        assert_eq!(state.search_text("QUASARCLAUSE", &owner, 5).len(), 1);
    }
