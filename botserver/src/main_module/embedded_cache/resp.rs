//! RESP2 framing for the built-in cache: commands in, replies out.

use std::io::{self, BufRead, Read};

const MAX_LINE: usize = 64 * 1024;
const MAX_ARGS: usize = 64 * 1024;
const MAX_BULK: usize = 16 * 1024 * 1024;
/// Before AUTH a client may only send short commands.
const PRE_AUTH_ARGS: usize = 8;
const PRE_AUTH_BULK: usize = 4 * 1024;

pub(super) enum Reply {
    Ok,
    Status(&'static str),
    Error(String),
    Int(i64),
    Bulk(Vec<u8>),
    Nil,
    Array(Vec<Reply>),
}

impl Reply {
    pub(super) fn error(text: impl Into<String>) -> Self {
        Self::Error(text.into())
    }

    pub(super) fn bulks(items: Vec<Vec<u8>>) -> Self {
        Self::Array(items.into_iter().map(Reply::Bulk).collect())
    }
}

/// Reads one command. `Ok(None)` is a clean end of stream; an empty vector is
/// a blank inline line; `InvalidData` is a protocol error.
pub(super) fn read_command<R: BufRead>(
    reader: &mut R,
    authenticated: bool,
) -> io::Result<Option<Vec<Vec<u8>>>> {
    let (max_args, max_bulk) = if authenticated {
        (MAX_ARGS, MAX_BULK)
    } else {
        (PRE_AUTH_ARGS, PRE_AUTH_BULK)
    };
    let Some(line) = read_line(reader)? else {
        return Ok(None);
    };
    let Some(count) = line.strip_prefix(b"*") else {
        let args: Vec<Vec<u8>> = line
            .split(u8::is_ascii_whitespace)
            .filter(|part| !part.is_empty())
            .map(<[u8]>::to_vec)
            .collect();
        if args.len() > max_args {
            return Err(protocol("too many arguments"));
        }
        return Ok(Some(args));
    };
    let count = parse_len(count)?;
    let Ok(count) = usize::try_from(count) else {
        return Ok(Some(Vec::new()));
    };
    if count > max_args {
        return Err(protocol("too many arguments"));
    }
    let mut args = Vec::with_capacity(count);
    for _ in 0..count {
        let header = read_line(reader)?.ok_or_else(|| protocol("unexpected end of stream"))?;
        let len = header
            .strip_prefix(b"$")
            .ok_or_else(|| protocol("expected '$'"))
            .and_then(parse_len)?;
        let len = usize::try_from(len).map_err(|_| protocol("invalid bulk length"))?;
        if len > max_bulk {
            return Err(protocol("invalid bulk length"));
        }
        let mut bulk = vec![0_u8; len + 2];
        reader.read_exact(&mut bulk)?;
        if !bulk.ends_with(b"\r\n") {
            return Err(protocol("bulk not terminated"));
        }
        bulk.truncate(len);
        args.push(bulk);
    }
    Ok(Some(args))
}

pub(super) fn encode(reply: &Reply, out: &mut Vec<u8>) {
    match reply {
        Reply::Ok => out.extend_from_slice(b"+OK\r\n"),
        Reply::Status(text) => line(out, b'+', text.as_bytes()),
        Reply::Error(text) => {
            let clean: Vec<u8> = text
                .bytes()
                .map(|b| if b.is_ascii_control() { b' ' } else { b })
                .collect();
            line(out, b'-', &clean);
        }
        Reply::Int(n) => line(out, b':', n.to_string().as_bytes()),
        Reply::Bulk(bytes) => {
            line(out, b'$', bytes.len().to_string().as_bytes());
            out.extend_from_slice(bytes);
            out.extend_from_slice(b"\r\n");
        }
        Reply::Nil => out.extend_from_slice(b"$-1\r\n"),
        Reply::Array(items) => {
            line(out, b'*', items.len().to_string().as_bytes());
            for item in items {
                encode(item, out);
            }
        }
    }
}

fn line(out: &mut Vec<u8>, prefix: u8, body: &[u8]) {
    out.push(prefix);
    out.extend_from_slice(body);
    out.extend_from_slice(b"\r\n");
}

fn read_line<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let read = reader
        .by_ref()
        .take(MAX_LINE as u64 + 2)
        .read_until(b'\n', &mut line)?;
    if read == 0 {
        return Ok(None);
    }
    if line.pop() != Some(b'\n') {
        return Err(protocol("line too long or truncated"));
    }
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Ok(Some(line))
}

fn parse_len(digits: &[u8]) -> io::Result<i64> {
    std::str::from_utf8(digits)
        .ok()
        .and_then(|text| text.parse::<i64>().ok())
        .ok_or_else(|| protocol("invalid length"))
}

fn protocol(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("Protocol error: {reason}"))
}
