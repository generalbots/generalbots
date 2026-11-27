//! A small cache server built into botserver, spoken to over the Redis
//! protocol (RESP2).
//!
//! It replaces an external Redis where the edition asks for it: it listens on
//! 127.0.0.1 only, on a free port picked by the system, and every connection
//! must AUTH with a password made for this process. The `redis::Client` the
//! rest of the server uses connects to it exactly as to a real Redis.
//!
//! Each connection runs on its own thread, so blocking and async callers both
//! work. Data lives in memory only, within a bound (see `store`). The server
//! lives as long as the process.

mod commands;
mod keyspace;
mod resp;
mod store;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::io::{self, BufReader, BufWriter, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::thread;
use std::time::Duration;

use rand::RngCore;

use commands::Session;
use store::Store;

const MAX_CLIENTS: usize = 256;
const PASSWORD_BYTES: usize = 24;

static PROCESS_CACHE: OnceLock<BuiltInCache> = OnceLock::new();

/// Starts the process-wide built-in cache once and returns its connection URL.
pub fn start_process_cache() -> io::Result<String> {
    if let Some(cache) = PROCESS_CACHE.get() {
        return Ok(cache.url());
    }
    let cache = BuiltInCache::start(store::MEMORY_CAP)?;
    let url = cache.url();
    // A concurrent caller that lost the race drops (and stops) its own server.
    let _ = PROCESS_CACHE.set(cache);
    Ok(PROCESS_CACHE.get().map_or(url, BuiltInCache::url))
}

struct Shared {
    store: Mutex<Store>,
    password: String,
    unsupported: Mutex<HashSet<String>>,
    stopping: AtomicBool,
    connections: Mutex<HashMap<u64, TcpStream>>,
    next_id: AtomicU64,
}

impl Shared {
    fn store(&self) -> MutexGuard<'_, Store> {
        self.store.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn connections(&self) -> MutexGuard<'_, HashMap<u64, TcpStream>> {
        self.connections.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A running built-in cache; dropping it stops the listener and closes its connections.
struct BuiltInCache {
    shared: Arc<Shared>,
    addr: SocketAddr,
}

impl BuiltInCache {
    fn start(memory_cap: usize) -> io::Result<Self> {
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))?;
        let addr = listener.local_addr()?;
        let shared = Arc::new(Shared {
            store: Mutex::new(Store::new(memory_cap)),
            password: new_password(),
            unsupported: Mutex::new(HashSet::new()),
            stopping: AtomicBool::new(false),
            connections: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        });
        let accepting = Arc::clone(&shared);
        thread::Builder::new()
            .name("builtin-cache".into())
            .spawn(move || accept_loop(&listener, &accepting))?;
        log::info!("Built-in cache listening on {addr}");
        Ok(Self { shared, addr })
    }

    fn url(&self) -> String {
        format!("redis://:{}@{}", self.shared.password, self.addr)
    }
}

impl Drop for BuiltInCache {
    fn drop(&mut self) {
        self.shared.stopping.store(true, Ordering::SeqCst);
        for stream in self.shared.connections().values() {
            let _ = stream.shutdown(Shutdown::Both);
        }
        // Wakes the accept loop so it sees the stop flag.
        let _ = TcpStream::connect_timeout(&self.addr, Duration::from_millis(200));
    }
}

fn new_password() -> String {
    let mut bytes = [0_u8; PASSWORD_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn accept_loop(listener: &TcpListener, shared: &Arc<Shared>) {
    for incoming in listener.incoming() {
        if shared.stopping.load(Ordering::SeqCst) {
            break;
        }
        let stream = match incoming {
            Ok(stream) => stream,
            Err(e) => {
                log::debug!("Built-in cache accept failed: {e}");
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        };
        let id = shared.next_id.fetch_add(1, Ordering::Relaxed);
        if !register(shared, id, &stream) {
            let _ = (&stream).write_all(b"-ERR max number of clients reached\r\n");
            continue;
        }
        let serving = Arc::clone(shared);
        let spawned = thread::Builder::new()
            .name("builtin-cache-conn".into())
            .spawn(move || {
                serve(&serving, stream, id);
                serving.connections().remove(&id);
            });
        if let Err(e) = spawned {
            log::warn!("Built-in cache could not start a connection thread: {e}");
            shared.connections().remove(&id);
        }
    }
}

fn register(shared: &Shared, id: u64, stream: &TcpStream) -> bool {
    let Ok(handle) = stream.try_clone() else {
        return false;
    };
    let mut connections = shared.connections();
    if connections.len() >= MAX_CLIENTS {
        return false;
    }
    connections.insert(id, handle);
    true
}

fn serve(shared: &Shared, stream: TcpStream, id: u64) {
    let _ = stream.set_nodelay(true);
    let Ok(write_half) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(stream);
    let mut writer = BufWriter::new(write_half);
    let mut session = Session::new(id);
    let mut out = Vec::new();
    while !shared.stopping.load(Ordering::SeqCst) {
        let args = match resp::read_command(&mut reader, session.authenticated) {
            Ok(Some(args)) => args,
            Ok(None) => break,
            Err(e) => {
                if e.kind() == io::ErrorKind::InvalidData {
                    out.clear();
                    resp::encode(&resp::Reply::Error(format!("ERR {e}")), &mut out);
                    let _ = writer.write_all(&out).and_then(|()| writer.flush());
                }
                break;
            }
        };
        if args.is_empty() {
            continue;
        }
        let (reply, close) = commands::execute(shared, &mut session, &args);
        out.clear();
        resp::encode(&reply, &mut out);
        if writer.write_all(&out).and_then(|()| writer.flush()).is_err() || close {
            break;
        }
    }
}
