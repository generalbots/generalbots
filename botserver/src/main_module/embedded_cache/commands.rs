//! Command dispatch of the built-in cache and the connection commands.

use std::sync::PoisonError;

use super::keyspace;
use super::resp::Reply;
use super::Shared;

/// The version reported to clients that ask; no call site depends on it.
const REPORTED_VERSION: &str = "7.0.0";
const MAX_WARNED_COMMANDS: usize = 64;

pub(super) struct Session {
    pub(super) authenticated: bool,
    name: Option<Vec<u8>>,
    id: u64,
}

impl Session {
    pub(super) fn new(id: u64) -> Self {
        Self {
            authenticated: false,
            name: None,
            id,
        }
    }
}

/// Runs one command; the flag is true when the connection must close after the reply.
pub(super) fn execute(shared: &Shared, session: &mut Session, args: &[Vec<u8>]) -> (Reply, bool) {
    let Some((name, rest)) = args.split_first() else {
        return (Reply::error("ERR empty command"), false);
    };
    let command = printable(name).to_ascii_uppercase();
    if !session.authenticated && !matches!(command.as_str(), "AUTH" | "HELLO" | "QUIT") {
        return (Reply::error("NOAUTH Authentication required."), false);
    }
    let reply = match command.as_str() {
        "AUTH" => match rest {
            [password] => check_credentials(shared, session, None, password),
            [user, password] => check_credentials(shared, session, Some(user), password),
            _ => arity("auth"),
        },
        "HELLO" => hello(shared, session, rest),
        "QUIT" => return (Reply::Ok, true),
        "PING" => match rest {
            [] => Reply::Status("PONG"),
            [message] => Reply::Bulk(message.clone()),
            _ => arity("ping"),
        },
        "ECHO" => match rest {
            [message] => Reply::Bulk(message.clone()),
            _ => arity("echo"),
        },
        "SELECT" => match rest {
            [index] => match parse_int(index) {
                Some(0..=15) => Reply::Ok,
                Some(_) => Reply::error("ERR DB index is out of range"),
                None => not_integer(),
            },
            _ => arity("select"),
        },
        "CLIENT" => client(session, rest),
        "COMMAND" => Reply::Array(Vec::new()),
        "INFO" => info(shared),
        "PUBLISH" => match rest {
            [_, _] => Reply::Int(0),
            _ => arity("publish"),
        },
        _ => match keyspace::execute(shared, &command, rest) {
            Some(reply) => reply,
            None => unsupported(shared, &command),
        },
    };
    (reply, false)
}

fn check_credentials(
    shared: &Shared,
    session: &mut Session,
    user: Option<&Vec<u8>>,
    password: &[u8],
) -> Reply {
    let user_ok = user.is_none_or(|name| name.as_slice() == b"default");
    if user_ok && same_secret(password, shared.password.as_bytes()) {
        session.authenticated = true;
        Reply::Ok
    } else {
        Reply::error("WRONGPASS invalid username-password pair or user is disabled.")
    }
}

fn hello(shared: &Shared, session: &mut Session, args: &[Vec<u8>]) -> Reply {
    let mut rest = args;
    if let Some((version, tail)) = rest.split_first() {
        match version.as_slice() {
            b"2" => {}
            b"3" => return Reply::error("NOPROTO unsupported protocol version"),
            _ => return Reply::error("ERR Protocol version is not an integer or out of range"),
        }
        rest = tail;
    }
    while let Some((option, tail)) = rest.split_first() {
        if option.eq_ignore_ascii_case(b"AUTH") {
            let [user, password, after @ ..] = tail else {
                return syntax();
            };
            if let Reply::Error(text) = check_credentials(shared, session, Some(user), password) {
                return Reply::Error(text);
            }
            rest = after;
        } else if option.eq_ignore_ascii_case(b"SETNAME") {
            let [name, after @ ..] = tail else {
                return syntax();
            };
            session.name = Some(name.clone());
            rest = after;
        } else {
            return syntax();
        }
    }
    if !session.authenticated {
        return Reply::error(
            "NOAUTH HELLO must be called with the client already authenticated, otherwise the \
             HELLO <proto> AUTH <user> <pass> option can be used",
        );
    }
    let text = |value: &str| Reply::Bulk(value.as_bytes().to_vec());
    Reply::Array(vec![
        text("server"),
        text("redis"),
        text("version"),
        text(REPORTED_VERSION),
        text("proto"),
        Reply::Int(2),
        text("id"),
        Reply::Int(i64::try_from(session.id).unwrap_or(i64::MAX)),
        text("mode"),
        text("standalone"),
        text("role"),
        text("master"),
        text("modules"),
        Reply::Array(Vec::new()),
    ])
}

fn client(session: &mut Session, args: &[Vec<u8>]) -> Reply {
    let Some((sub, tail)) = args.split_first() else {
        return arity("client");
    };
    match printable(sub).to_ascii_uppercase().as_str() {
        "SETINFO" => match tail {
            [_, _] => Reply::Ok,
            _ => arity("client|setinfo"),
        },
        "SETNAME" => match tail {
            [name] => {
                session.name = Some(name.clone());
                Reply::Ok
            }
            _ => arity("client|setname"),
        },
        "GETNAME" => session.name.clone().map_or(Reply::Nil, Reply::Bulk),
        "ID" => Reply::Int(i64::try_from(session.id).unwrap_or(i64::MAX)),
        other => Reply::error(format!("ERR unknown subcommand '{other}'")),
    }
}

fn info(shared: &Shared) -> Reply {
    let mut store = shared.store();
    let keys = store.len();
    let used = store.used();
    let body = format!(
        "# Server\r\nredis_version:{REPORTED_VERSION}\r\nredis_mode:standalone\r\n\r\n\
         # Memory\r\nused_memory:{used}\r\nmaxmemory:{}\r\n\r\n\
         # Keyspace\r\ndb0:keys={keys},expires=0,avg_ttl=0\r\n",
        super::store::MEMORY_CAP
    );
    Reply::Bulk(body.into_bytes())
}

/// Unknown commands get an error reply; each name is logged once so gaps show.
fn unsupported(shared: &Shared, command: &str) -> Reply {
    let mut seen = shared
        .unsupported
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if seen.len() < MAX_WARNED_COMMANDS && seen.insert(command.to_string()) {
        log::warn!("Built-in cache: unsupported command {command}");
    }
    Reply::error(format!("ERR unknown command '{command}'"))
}

/// A short, log-safe rendering of client input.
fn printable(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(32)
        .map(|&b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'|') {
                char::from(b)
            } else {
                '?'
            }
        })
        .collect()
}

fn same_secret(given: &[u8], expected: &[u8]) -> bool {
    given.len() == expected.len()
        && given
            .iter()
            .zip(expected)
            .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

pub(super) fn parse_int(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

pub(super) fn arity(command: &str) -> Reply {
    Reply::error(format!("ERR wrong number of arguments for '{command}' command"))
}

pub(super) fn syntax() -> Reply {
    Reply::error("ERR syntax error")
}

pub(super) fn not_integer() -> Reply {
    Reply::error("ERR value is not an integer or out of range")
}

pub(super) fn wrong_type() -> Reply {
    Reply::error("WRONGTYPE Operation against a key holding the wrong kind of value")
}

pub(super) fn count(n: usize) -> Reply {
    Reply::Int(i64::try_from(n).unwrap_or(i64::MAX))
}
