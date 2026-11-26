//! Data commands of the built-in cache: strings, expiry, key walks, sets, hashes.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use super::commands::{arity, count, not_integer, parse_int, syntax, wrong_type};
use super::resp::Reply;
use super::store::{Entry, Store, Value};
use super::Shared;

const SCAN_DEFAULT_COUNT: usize = 10;
const SECONDS: u64 = 1000;
const MILLIS: u64 = 1;

/// Runs a data command; None when `command` is not one.
pub(super) fn execute(shared: &Shared, command: &str, args: &[Vec<u8>]) -> Option<Reply> {
    let mut guard = shared.store();
    let store = &mut *guard;
    store.tick();
    let reply = match command {
        "GET" => get(store, args),
        "SET" => set(store, args),
        "SETEX" => set_with_ttl(store, args, "setex", SECONDS),
        "PSETEX" => set_with_ttl(store, args, "psetex", MILLIS),
        "DEL" | "UNLINK" if !args.is_empty() => count(args.iter().filter(|k| store.take(k).is_some()).count()),
        "EXISTS" if !args.is_empty() => count(args.iter().filter(|k| store.get(k).is_some()).count()),
        "DEL" | "UNLINK" | "EXISTS" => arity(&command.to_ascii_lowercase()),
        "EXPIRE" => expire(store, args, "expire", SECONDS),
        "PEXPIRE" => expire(store, args, "pexpire", MILLIS),
        "TTL" => ttl(store, args, SECONDS),
        "PTTL" => ttl(store, args, MILLIS),
        "PERSIST" => match args {
            [key] => {
                let had_expiry = store.get(key).is_some_and(|e| e.expires_at.is_some());
                count(usize::from(had_expiry && store.set_expiry(key, None)))
            }
            _ => arity("persist"),
        },
        "KEYS" => match args {
            [pattern] => Reply::bulks(store.matching_keys(pattern)),
            _ => arity("keys"),
        },
        "SCAN" => scan(store, args),
        "DBSIZE" => count(store.len()),
        "FLUSHDB" | "FLUSHALL" => {
            store.clear();
            Reply::Ok
        }
        "SADD" => sadd(store, args),
        "SREM" => srem(store, args),
        "SMEMBERS" => smembers(store, args),
        "HSET" => hset(store, args),
        "HGET" => hget(store, args),
        "HGETALL" => hgetall(store, args),
        "HDEL" => hdel(store, args),
        _ => return None,
    };
    Some(reply)
}

fn deadline(amount: &[u8], unit_ms: u64, command: &str) -> Result<Instant, Reply> {
    let invalid = || Reply::error(format!("ERR invalid expire time in '{command}' command"));
    let amount = parse_int(amount).ok_or_else(not_integer)?;
    let millis = u64::try_from(amount)
        .ok()
        .filter(|n| *n > 0)
        .and_then(|n| n.checked_mul(unit_ms))
        .ok_or_else(invalid)?;
    Instant::now()
        .checked_add(Duration::from_millis(millis))
        .ok_or_else(invalid)
}

fn get(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key] = args else {
        return arity("get");
    };
    match store.get(key).map(|entry| &entry.value) {
        None => Reply::Nil,
        Some(Value::Str(bytes)) => Reply::Bulk(bytes.clone()),
        Some(_) => wrong_type(),
    }
}

fn set(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key, value, options @ ..] = args else {
        return arity("set");
    };
    let (mut only_new, mut only_existing, mut keep_ttl) = (false, false, false);
    let mut expires_at = None;
    let mut rest = options;
    while let Some((option, tail)) = rest.split_first() {
        rest = tail;
        if option.eq_ignore_ascii_case(b"NX") {
            only_new = true;
        } else if option.eq_ignore_ascii_case(b"XX") {
            only_existing = true;
        } else if option.eq_ignore_ascii_case(b"KEEPTTL") {
            keep_ttl = true;
        } else if option.eq_ignore_ascii_case(b"EX") || option.eq_ignore_ascii_case(b"PX") {
            let Some((amount, after)) = tail.split_first() else {
                return syntax();
            };
            let unit = if option.eq_ignore_ascii_case(b"EX") { SECONDS } else { MILLIS };
            match deadline(amount, unit, "set") {
                Ok(at) => expires_at = Some(at),
                Err(reply) => return reply,
            }
            rest = after;
        } else {
            return syntax();
        }
    }
    if (only_new && only_existing) || (keep_ttl && expires_at.is_some()) {
        return syntax();
    }
    let current = store.get(key).map(|entry| entry.expires_at);
    if (only_new && current.is_some()) || (only_existing && current.is_none()) {
        return Reply::Nil;
    }
    if keep_ttl {
        expires_at = current.flatten();
    }
    store.put(key.clone(), Entry::new(Value::Str(value.clone()), expires_at));
    Reply::Ok
}

fn set_with_ttl(store: &mut Store, args: &[Vec<u8>], command: &str, unit: u64) -> Reply {
    let [key, amount, value] = args else {
        return arity(command);
    };
    match deadline(amount, unit, command) {
        Ok(at) => {
            store.put(key.clone(), Entry::new(Value::Str(value.clone()), Some(at)));
            Reply::Ok
        }
        Err(reply) => reply,
    }
}

fn expire(store: &mut Store, args: &[Vec<u8>], command: &str, unit: u64) -> Reply {
    let [key, amount] = args else {
        return arity(command);
    };
    match parse_int(amount) {
        None => not_integer(),
        Some(n) if n <= 0 => count(usize::from(store.take(key).is_some())),
        Some(_) => match deadline(amount, unit, command) {
            Ok(at) => count(usize::from(store.set_expiry(key, Some(at)))),
            Err(reply) => reply,
        },
    }
}

fn ttl(store: &mut Store, args: &[Vec<u8>], unit: u64) -> Reply {
    let [key] = args else {
        return arity(if unit == SECONDS { "ttl" } else { "pttl" });
    };
    let remaining = match store.get(key) {
        None => return Reply::Int(-2),
        Some(Entry { expires_at: None, .. }) => return Reply::Int(-1),
        Some(Entry { expires_at: Some(at), .. }) => at.saturating_duration_since(Instant::now()),
    };
    let millis = remaining.as_millis();
    // Round the same way Redis does in ttlGenericCommand (expire.c).
    let value = if unit == SECONDS { (millis + 500) / 1000 } else { millis };
    Reply::Int(i64::try_from(value).unwrap_or(i64::MAX))
}

fn scan(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let Some((cursor, mut rest)) = args.split_first() else {
        return arity("scan");
    };
    let Some(cursor) = std::str::from_utf8(cursor).ok().and_then(|c| c.parse::<u64>().ok()) else {
        return Reply::error("ERR invalid cursor");
    };
    let mut pattern = None;
    let mut batch = SCAN_DEFAULT_COUNT;
    while let [option, value, after @ ..] = rest {
        if option.eq_ignore_ascii_case(b"MATCH") {
            pattern = Some(value.as_slice());
        } else if option.eq_ignore_ascii_case(b"COUNT") {
            match parse_int(value).and_then(|n| usize::try_from(n).ok()) {
                Some(n) if n > 0 => batch = n,
                _ => return syntax(),
            }
        } else {
            return syntax();
        }
        rest = after;
    }
    if !rest.is_empty() {
        return syntax();
    }
    let (next, keys) = store.scan(cursor, pattern, batch);
    Reply::Array(vec![Reply::Bulk(next.to_string().into_bytes()), Reply::bulks(keys)])
}

type Members = HashSet<Vec<u8>>;
type Fields = HashMap<Vec<u8>, Vec<u8>>;

fn take_set(store: &mut Store, key: &[u8]) -> Result<(Members, Option<Instant>), Reply> {
    if matches!(store.get(key), Some(entry) if !matches!(entry.value, Value::Set(_))) {
        return Err(wrong_type());
    }
    Ok(match store.take(key) {
        Some(Entry { value: Value::Set(members), expires_at }) => (members, expires_at),
        _ => (Members::new(), None),
    })
}

fn take_hash(store: &mut Store, key: &[u8]) -> Result<(Fields, Option<Instant>), Reply> {
    if matches!(store.get(key), Some(entry) if !matches!(entry.value, Value::Hash(_))) {
        return Err(wrong_type());
    }
    Ok(match store.take(key) {
        Some(Entry { value: Value::Hash(fields), expires_at }) => (fields, expires_at),
        _ => (Fields::new(), None),
    })
}

fn sadd(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key, members @ ..] = args else {
        return arity("sadd");
    };
    if members.is_empty() {
        return arity("sadd");
    }
    let (mut set, expires_at) = match take_set(store, key) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let added = members.iter().filter(|m| set.insert((*m).clone())).count();
    store.put(key.clone(), Entry::new(Value::Set(set), expires_at));
    count(added)
}

fn srem(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key, members @ ..] = args else {
        return arity("srem");
    };
    if members.is_empty() {
        return arity("srem");
    }
    let (mut set, expires_at) = match take_set(store, key) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let removed = members.iter().filter(|m| set.remove(m.as_slice())).count();
    if !set.is_empty() {
        store.put(key.clone(), Entry::new(Value::Set(set), expires_at));
    }
    count(removed)
}

fn smembers(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key] = args else {
        return arity("smembers");
    };
    match store.get(key).map(|entry| &entry.value) {
        None => Reply::Array(Vec::new()),
        Some(Value::Set(members)) => Reply::bulks(members.iter().cloned().collect()),
        Some(_) => wrong_type(),
    }
}

fn hset(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key, pairs @ ..] = args else {
        return arity("hset");
    };
    if pairs.is_empty() || pairs.len() % 2 != 0 {
        return arity("hset");
    }
    let (mut fields, expires_at) = match take_hash(store, key) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let added = pairs
        .chunks_exact(2)
        .filter(|pair| match pair {
            [field, value] => fields.insert(field.clone(), value.clone()).is_none(),
            _ => false,
        })
        .count();
    store.put(key.clone(), Entry::new(Value::Hash(fields), expires_at));
    count(added)
}

fn hget(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key, field] = args else {
        return arity("hget");
    };
    match store.get(key).map(|entry| &entry.value) {
        None => Reply::Nil,
        Some(Value::Hash(fields)) => fields.get(field).cloned().map_or(Reply::Nil, Reply::Bulk),
        Some(_) => wrong_type(),
    }
}

fn hgetall(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key] = args else {
        return arity("hgetall");
    };
    match store.get(key).map(|entry| &entry.value) {
        None => Reply::Array(Vec::new()),
        Some(Value::Hash(fields)) => Reply::bulks(
            fields
                .iter()
                .flat_map(|(field, value)| [field.clone(), value.clone()])
                .collect(),
        ),
        Some(_) => wrong_type(),
    }
}

fn hdel(store: &mut Store, args: &[Vec<u8>]) -> Reply {
    let [key, names @ ..] = args else {
        return arity("hdel");
    };
    if names.is_empty() {
        return arity("hdel");
    }
    let (mut fields, expires_at) = match take_hash(store, key) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let removed = names.iter().filter(|name| fields.remove(name.as_slice()).is_some()).count();
    if !fields.is_empty() {
        store.put(key.clone(), Entry::new(Value::Hash(fields), expires_at));
    }
    count(removed)
}
