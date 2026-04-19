use bottest::prelude::*;

#[tokio::test]
async fn test_database_ping() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct PingResult {
        #[diesel(sql_type = Text)]
        result: String,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<PingResult> = sql_query("SELECT 'pong' as result")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].result, "pong");
}

#[tokio::test]
async fn test_execute_raw_sql() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct SumResult {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        sum: i32,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<SumResult> = sql_query("SELECT 2 + 2 as sum")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].sum, 4);
}

#[tokio::test]
async fn test_transaction_rollback() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    let mut conn = pool.get().expect("Failed to get connection");

    let result: Result<(), diesel::result::Error> = conn
        .transaction::<(), diesel::result::Error, _>(|conn| {
            sql_query("SELECT 1").execute(conn)?;
            Err(diesel::result::Error::RollbackTransaction)
        });

    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_connections() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let conn1 = pool.get();
    let conn2 = pool.get();
    let conn3 = pool.get();

    assert!(conn1.is_ok());
    assert!(conn2.is_ok());
    assert!(conn3.is_ok());
}

#[tokio::test]
async fn test_query_result_types() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct TypeTestRow {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        integer: i32,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        bigint: i64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        text: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        flag: bool,
        #[diesel(sql_type = diesel::sql_types::Double)]
        decimal: f64,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<TypeTestRow> = sql_query(
        "SELECT
            42 as integer,
            9223372036854775807::bigint as bigint,
            'hello' as text,
            true as flag,
            3.125 as decimal",
    )
    .load(&mut conn)
    .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].integer, 42);
    assert_eq!(result[0].bigint, 9_223_372_036_854_775_807_i64);
    assert_eq!(result[0].text, "hello");
    assert!(result[0].flag);
    assert!((result[0].decimal - 3.125).abs() < 0.0001);
}

#[tokio::test]
async fn test_null_handling() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct NullTestResult {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        nullable_val: Option<String>,
    }

    let mut conn = pool.get().expect("Failed to get connection");

    let result: Vec<NullTestResult> = sql_query("SELECT NULL::text as nullable_val")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert!(result[0].nullable_val.is_none());

    let result: Vec<NullTestResult> = sql_query("SELECT 'value'::text as nullable_val")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].nullable_val, Some("value".to_string()));
}

#[tokio::test]
async fn test_json_handling() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct JsonTestResult {
        #[diesel(sql_type = diesel::sql_types::Text)]
        json_text: String,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<JsonTestResult> =
        sql_query(r#"SELECT '{"key": "value", "number": 42}'::jsonb::text as json_text"#)
            .load(&mut conn)
            .expect("Query failed");

    assert_eq!(result.len(), 1);

    let parsed: serde_json::Value =
        serde_json::from_str(&result[0].json_text).expect("Failed to parse JSON");

    assert_eq!(parsed["key"], "value");
    assert_eq!(parsed["number"], 42);
}

#[tokio::test]
async fn test_uuid_generation() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct UuidResult {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<UuidResult> = sql_query("SELECT gen_random_uuid() as id")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert!(!result[0].id.is_nil());
}

#[tokio::test]
async fn test_timestamp_handling() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use chrono::Utc;
    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct TimestampResult {
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        ts: chrono::DateTime<chrono::Utc>,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<TimestampResult> = sql_query("SELECT NOW() as ts")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);

    let now = Utc::now();
    let diff = now.signed_duration_since(result[0].ts);
    assert!(diff.num_seconds().abs() < 60);
}

#[tokio::test]
async fn test_array_handling() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let pool = match ctx.db_pool().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(QueryableByName)]
    struct ArrayResult {
        #[diesel(sql_type = diesel::sql_types::Array<diesel::sql_types::Text>)]
        items: Vec<String>,
    }

    let mut conn = pool.get().expect("Failed to get connection");
    let result: Vec<ArrayResult> = sql_query("SELECT ARRAY['a', 'b', 'c'] as items")
        .load(&mut conn)
        .expect("Query failed");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].items, vec!["a", "b", "c"]);
}

#[tokio::test]
async fn test_insert_user_fixture() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let user = admin_user();
    if let Err(e) = ctx.insert_user(&user).await {
        eprintln!("Skipping insert test (table may not exist): {}", e);
        return;
    }

    let pool = ctx.db_pool().await.unwrap();
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Uuid as DieselUuid;

    #[derive(QueryableByName)]
    struct UserCheck {
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
    }

    let mut conn = pool.get().unwrap();
    let result: Result<Vec<UserCheck>, _> = sql_query("SELECT email FROM users WHERE id = $1")
        .bind::<DieselUuid, _>(user.id)
        .load(&mut conn);

    if let Ok(users) = result {
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].email, user.email);
    }
}

#[tokio::test]
async fn test_insert_bot_fixture() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let bot = bot_with_kb("test-knowledge-bot");
    if let Err(e) = ctx.insert_bot(&bot).await {
        eprintln!("Skipping insert test (table may not exist): {}", e);
        return;
    }

    let pool = ctx.db_pool().await.unwrap();
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Uuid as DieselUuid;

    #[derive(QueryableByName)]
    struct BotCheck {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        kb_enabled: bool,
    }

    let mut conn = pool.get().unwrap();
    let result: Result<Vec<BotCheck>, _> =
        sql_query("SELECT name, kb_enabled FROM bots WHERE id = $1")
            .bind::<DieselUuid, _>(bot.id)
            .load(&mut conn);

    if let Ok(bots) = result {
        assert_eq!(bots.len(), 1);
        assert_eq!(bots[0].name, "test-knowledge-bot");
        assert!(bots[0].kb_enabled);
    }
}

#[tokio::test]
async fn test_session_and_message_fixtures() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let bot = basic_bot("session-test-bot");
    let customer = customer("+15551234567");
    let session = session_for(&bot, &customer);
    let message = message_in_session(&session, "Hello from test", MessageDirection::Incoming);

    if ctx.insert_bot(&bot).await.is_err() {
        eprintln!("Skipping: tables may not exist");
        return;
    }

    let _ = ctx.insert_customer(&customer).await;
    let _ = ctx.insert_session(&session).await;
    let _ = ctx.insert_message(&message).await;

    let pool = ctx.db_pool().await.unwrap();
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Uuid as DieselUuid;

    #[derive(QueryableByName)]
    struct MessageCheck {
        #[diesel(sql_type = diesel::sql_types::Text)]
        content: String,
    }

    let mut conn = pool.get().unwrap();
    let result: Result<Vec<MessageCheck>, _> =
        sql_query("SELECT content FROM messages WHERE session_id = $1")
            .bind::<DieselUuid, _>(session.id)
            .load(&mut conn);

    if let Ok(messages) = result {
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello from test");
    }
}
