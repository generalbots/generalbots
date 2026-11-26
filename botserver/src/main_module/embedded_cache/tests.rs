//! The built-in cache spoken to by the real `redis` client, blocking and async.

use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use super::store::{glob_match, Entry, Store, Value};
use super::BuiltInCache;

fn cache() -> BuiltInCache {
    match BuiltInCache::start(1024 * 1024) {
        Ok(cache) => cache,
        Err(e) => panic!("built-in cache did not start: {e}"),
    }
}

fn connect(url: &str) -> redis::Connection {
    match redis::Client::open(url).and_then(|client| client.get_connection()) {
        Ok(conn) => conn,
        Err(e) => panic!("connection failed: {e}"),
    }
}

async fn connect_async(url: &str) -> redis::aio::MultiplexedConnection {
    let client = match redis::Client::open(url) {
        Ok(client) => client,
        Err(e) => panic!("client failed: {e}"),
    };
    match client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => panic!("async connection failed: {e}"),
    }
}

fn query<T: redis::FromRedisValue>(cmd: &mut redis::Cmd, conn: &mut redis::Connection) -> T {
    match cmd.query(conn) {
        Ok(value) => value,
        Err(e) => panic!("command failed: {e}"),
    }
}

#[test]
fn auth_is_required_and_checked() {
    let cache = cache();
    let mut anonymous = connect(&format!("redis://{}", cache.addr));
    let refused = redis::cmd("GET").arg("k").query::<Option<String>>(&mut anonymous);
    assert_eq!(refused.err().and_then(|e| e.code().map(str::to_string)).as_deref(), Some("NOAUTH"));
    let refused_ping = redis::cmd("PING").query::<String>(&mut anonymous);
    assert!(refused_ping.is_err());

    let wrong = redis::Client::open(format!("redis://:wrong@{}", cache.addr).as_str())
        .and_then(|client| client.get_connection());
    assert!(wrong.is_err());

    let mut with_user = connect(&format!("redis://default:{}@{}", cache.shared.password, cache.addr));
    assert_eq!(query::<String>(&mut redis::cmd("PING"), &mut with_user), "PONG");

    let mut late = connect(&format!("redis://{}", cache.addr));
    let auth: String = query(redis::cmd("AUTH").arg(&cache.shared.password), &mut late);
    assert_eq!(auth, "OK");
    assert_eq!(query::<String>(&mut redis::cmd("PING"), &mut late), "PONG");
}

#[test]
fn set_nx_ex_answers_nil_the_second_time() {
    let cache = cache();
    let mut conn = connect(&cache.url());
    let set = |conn: &mut redis::Connection| -> Option<String> {
        query(redis::cmd("SET").arg("dedup").arg("1").arg("NX").arg("EX").arg(600_i64), conn)
    };
    assert_eq!(set(&mut conn).as_deref(), Some("OK"));
    assert_eq!(set(&mut conn), None);
    let ttl: i64 = query(redis::cmd("TTL").arg("dedup"), &mut conn);
    assert!((599..=600).contains(&ttl), "ttl {ttl}");
    let plain: i64 = query(redis::cmd("TTL").arg("missing"), &mut conn);
    assert_eq!(plain, -2);
}

#[test]
fn setex_and_px_keys_expire() {
    let cache = cache();
    let mut conn = connect(&cache.url());
    let _: () = query(redis::cmd("SETEX").arg("token").arg(30).arg("v"), &mut conn);
    let ttl: i64 = query(redis::cmd("TTL").arg("token"), &mut conn);
    assert!((29..=30).contains(&ttl), "ttl {ttl}");
    let _: () = query(redis::cmd("SET").arg("short").arg("v").arg("PX").arg(60), &mut conn);
    let _: () = query(redis::cmd("SET").arg("forever").arg("v"), &mut conn);
    assert_eq!(query::<i64>(redis::cmd("TTL").arg("forever"), &mut conn), -1);
    thread::sleep(Duration::from_millis(120));
    assert_eq!(query::<Option<String>>(redis::cmd("GET").arg("short"), &mut conn), None);
    assert_eq!(query::<i64>(redis::cmd("EXISTS").arg("short").arg("forever"), &mut conn), 1);
    assert_eq!(query::<i64>(redis::cmd("EXPIRE").arg("forever").arg(100), &mut conn), 1);
    assert!(query::<i64>(redis::cmd("PTTL").arg("forever"), &mut conn) > 99_000);
    let bad = redis::cmd("SETEX").arg("k").arg(0).arg("v").query::<()>(&mut conn);
    assert!(bad.is_err());
}

#[test]
fn keys_and_scan_follow_patterns() {
    let cache = cache();
    let mut conn = connect(&cache.url());
    for key in ["jwt_blacklist:a", "jwt_blacklist:b", "jwt_blacklist:c", "other:a", "sugg[1]"] {
        let _: () = query(redis::cmd("SET").arg(key).arg("1"), &mut conn);
    }
    let mut keys: Vec<String> = query(redis::cmd("KEYS").arg("jwt_blacklist:*"), &mut conn);
    keys.sort();
    assert_eq!(keys, ["jwt_blacklist:a", "jwt_blacklist:b", "jwt_blacklist:c"]);
    let escaped: Vec<String> = query(redis::cmd("KEYS").arg("sugg\\[1]"), &mut conn);
    assert_eq!(escaped, ["sugg[1]"]);

    let mut seen = HashSet::new();
    let mut cursor = 0_u64;
    let mut steps = 0;
    loop {
        let (next, batch): (u64, Vec<String>) = query(
            redis::cmd("SCAN").arg(cursor).arg("MATCH").arg("jwt_blacklist:*").arg("COUNT").arg(2),
            &mut conn,
        );
        for key in batch {
            if key == "jwt_blacklist:a" {
                let _: i64 = query(redis::cmd("DEL").arg("jwt_blacklist:c"), &mut conn);
            }
            seen.insert(key);
        }
        steps += 1;
        if next == 0 || steps > 10 {
            break;
        }
        cursor = next;
    }
    assert!(seen.contains("jwt_blacklist:a") && seen.contains("jwt_blacklist:b"));
    assert!(seen.iter().all(|k| k.starts_with("jwt_blacklist:")));
    assert!((2..=3).contains(&steps), "steps {steps}");
}

/// Drives SCAN with an `i64` cursor the same way `redis_blacklist_store.rs`
/// does, over enough keys that more than half of the old u64 hashes would
/// have exceeded `i64::MAX`, and deletes keys mid-walk.
#[test]
fn scan_i64_cursor_many_keys_with_delete() {
    let cache = cache();
    let mut conn = connect(&cache.url());
    let key_count = 200;
    for i in 0..key_count {
        let key = format!("jwt_blacklist:token_{i:04}");
        let _: () = query(redis::cmd("SETEX").arg(&key).arg(3600).arg("v"), &mut conn);
    }
    let _: () = query(redis::cmd("SET").arg("other_key").arg("v"), &mut conn);

    let mut seen = HashSet::new();
    let mut cursor: i64 = 0;
    let mut iterations = 0;
    loop {
        assert!(cursor >= 0, "cursor {cursor} is negative");
        let (next_cursor, keys): (i64, Vec<String>) = query(
            redis::cmd("SCAN").arg(cursor).arg("MATCH").arg("jwt_blacklist:*").arg("COUNT").arg(10),
            &mut conn,
        );
        assert!(next_cursor >= 0, "next_cursor {next_cursor} is negative");
        for key in &keys {
            assert!(key.starts_with("jwt_blacklist:"), "unexpected key {key}");
            if key.ends_with("0050") || key.ends_with("0100") || key.ends_with("0150") {
                let _: i64 = query(redis::cmd("DEL").arg(key.as_str()), &mut conn);
            }
        }
        seen.extend(keys);
        iterations += 1;
        cursor = next_cursor;
        if cursor == 0 {
            break;
        }
        assert!(iterations <= key_count, "SCAN did not terminate");
    }
    assert!(seen.len() >= key_count - 3, "seen {} keys, expected >= {}", seen.len(), key_count - 3);
    assert!(!seen.contains("other_key"));
}

#[test]
fn sets_and_hashes() {
    let cache = cache();
    let mut conn = connect(&cache.url());
    assert_eq!(query::<i64>(redis::cmd("SADD").arg("s").arg("a").arg("b").arg("a"), &mut conn), 2);
    let mut members: Vec<String> = query(redis::cmd("SMEMBERS").arg("s"), &mut conn);
    members.sort();
    assert_eq!(members, ["a", "b"]);
    assert_eq!(query::<Vec<String>>(redis::cmd("SMEMBERS").arg("none"), &mut conn), Vec::<String>::new());
    assert_eq!(query::<i64>(redis::cmd("HSET").arg("h").arg("ctx").arg("inactive"), &mut conn), 1);
    assert_eq!(query::<i64>(redis::cmd("HSET").arg("h").arg("ctx").arg("active"), &mut conn), 0);
    assert_eq!(query::<Option<String>>(redis::cmd("HGET").arg("h").arg("ctx"), &mut conn).as_deref(), Some("active"));
    let wrong = redis::cmd("GET").arg("s").query::<Option<String>>(&mut conn);
    assert_eq!(wrong.err().and_then(|e| e.code().map(str::to_string)).as_deref(), Some("WRONGTYPE"));
}

#[test]
fn connection_commands_answer_and_unknown_commands_fail() {
    let cache = cache();
    let mut conn = connect(&format!("{}/3", cache.url()));
    assert_eq!(query::<String>(&mut redis::cmd("PING"), &mut conn), "PONG");
    let setinfo: String = query(redis::cmd("CLIENT").arg("SETINFO").arg("LIB-NAME").arg("x"), &mut conn);
    assert_eq!(setinfo, "OK");
    assert_eq!(query::<String>(redis::cmd("CLIENT").arg("SETNAME").arg("n"), &mut conn), "OK");
    assert_eq!(query::<String>(redis::cmd("SELECT").arg(0), &mut conn), "OK");
    assert_eq!(query::<i64>(redis::cmd("PUBLISH").arg("ch").arg("m"), &mut conn), 0);
    assert!(query::<String>(&mut redis::cmd("INFO"), &mut conn).contains("redis_version"));
    let unknown = redis::cmd("XADD").arg("s").arg("*").arg("f").arg("v").query::<redis::Value>(&mut conn);
    assert!(unknown.is_err_and(|e| e.to_string().contains("unknown command")));
    assert_eq!(query::<String>(&mut redis::cmd("PING"), &mut conn), "PONG");
}

#[test]
fn concurrent_blocking_clients() {
    let cache = cache();
    let url = cache.url();
    let workers: Vec<_> = (0..8)
        .map(|worker| {
            let url = url.clone();
            thread::spawn(move || {
                for i in 0..50 {
                    let mut conn = connect(&url);
                    let key = format!("w{worker}:{i}");
                    let _: () = query(redis::cmd("SET").arg(&key).arg(i), &mut conn);
                    let back: i64 = query(redis::cmd("GET").arg(&key), &mut conn);
                    assert_eq!(back, i);
                }
            })
        })
        .collect();
    for worker in workers {
        assert!(worker.join().is_ok());
    }
    let mut conn = connect(&url);
    assert_eq!(query::<i64>(&mut redis::cmd("DBSIZE"), &mut conn), 400);
}

#[tokio::test]
async fn async_clients_share_the_keyspace() {
    let cache = cache();
    let mut first = connect_async(&cache.url()).await;
    let mut second = connect_async(&cache.url()).await;
    let set: Option<String> = redis::cmd("SET")
        .arg("session_init")
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(259_200)
        .query_async(&mut first)
        .await
        .unwrap_or_else(|e| panic!("async SET failed: {e}"));
    assert_eq!(set.as_deref(), Some("OK"));
    let again: Option<String> = redis::cmd("SET")
        .arg("session_init")
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(259_200)
        .query_async(&mut second)
        .await
        .unwrap_or_else(|e| panic!("async SET failed: {e}"));
    assert_eq!(again, None);
    let tasks: Vec<_> = (0..16)
        .map(|i| {
            let mut conn = first.clone();
            tokio::spawn(async move {
                redis::cmd("SADD")
                    .arg("suggestions")
                    .arg(i)
                    .query_async::<i64>(&mut conn)
                    .await
            })
        })
        .collect();
    for task in tasks {
        assert!(matches!(task.await, Ok(Ok(1))));
    }
    let members: Vec<String> = redis::cmd("SMEMBERS")
        .arg("suggestions")
        .query_async(&mut second)
        .await
        .unwrap_or_else(|e| panic!("async SMEMBERS failed: {e}"));
    assert_eq!(members.len(), 16);
    let unknown = redis::cmd("NOPE").query_async::<redis::Value>(&mut second).await;
    assert!(unknown.is_err());
    let pong: String = redis::cmd("PING")
        .query_async(&mut second)
        .await
        .unwrap_or_else(|e| panic!("async PING failed: {e}"));
    assert_eq!(pong, "PONG");
}

#[test]
fn memory_bound_evicts_expired_then_soonest_then_any() {
    let mut store = Store::new(2_000);
    let later = std::time::Instant::now() + Duration::from_secs(60);
    let soon = std::time::Instant::now() + Duration::from_secs(5);
    store.put(b"keep".to_vec(), Entry::new(Value::Str(vec![0; 400]), None));
    store.put(b"later".to_vec(), Entry::new(Value::Str(vec![0; 400]), Some(later)));
    store.put(b"soon".to_vec(), Entry::new(Value::Str(vec![0; 400]), Some(soon)));
    store.put(b"big".to_vec(), Entry::new(Value::Str(vec![0; 700]), None));
    assert!(store.get(b"soon").is_none());
    assert!(store.get(b"big").is_some() && store.get(b"keep").is_some());
    assert!(store.used() <= 2_000);
    store.put(b"huge".to_vec(), Entry::new(Value::Str(vec![0; 1_800]), None));
    assert!(store.get(b"huge").is_some());
    assert_eq!(store.len(), 1);
}

#[test]
fn glob_patterns() {
    assert!(glob_match(b"jwt_blacklist:*", b"jwt_blacklist:abc"));
    assert!(!glob_match(b"jwt_blacklist:*", b"jwt:abc"));
    assert!(glob_match(b"h?llo", b"hello") && !glob_match(b"h?llo", b"hllo"));
    assert!(glob_match(b"h[ae]llo", b"hallo") && !glob_match(b"h[^e]llo", b"hello"));
    assert!(glob_match(b"k[a-c]", b"kb") && !glob_match(b"k[a-c]", b"kd"));
    assert!(glob_match(b"*", b"") && glob_match(b"a\\*", b"a*") && !glob_match(b"a\\*", b"ab"));
}
