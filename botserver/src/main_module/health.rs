//! Health check, readiness and client error handlers.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use std::time::{Duration, Instant};

use botcore::shared::state::AppState;

/// Every dependency probe is bounded so `/ready` can never hang its caller.
const DEPENDENCY_TIMEOUT: Duration = Duration::from_millis(500);

/// One dependency's readiness result. Errors are logged, never returned to
/// clients: only the coarse status and the measured latency are exposed.
#[derive(serde::Serialize)]
struct DependencyProbe {
    status: &'static str,
    latency_ms: u64,
}

impl DependencyProbe {
    fn ok(started: Instant) -> Self {
        Self {
            status: "ok",
            latency_ms: elapsed_ms(started),
        }
    }

    fn fail(started: Instant) -> Self {
        Self {
            status: "fail",
            latency_ms: elapsed_ms(started),
        }
    }

    fn not_configured() -> Self {
        Self {
            status: "not_configured",
            latency_ms: 0,
        }
    }

    fn is_failed(&self) -> bool {
        self.status == "fail"
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u64::MAX as u128) as u64
}

fn from_outcome(ok: bool, started: Instant) -> DependencyProbe {
    if ok {
        DependencyProbe::ok(started)
    } else {
        DependencyProbe::fail(started)
    }
}

async fn check_postgres(state: &Arc<AppState>) -> DependencyProbe {
    let pool = state.conn.clone();
    let started = Instant::now();
    let outcome = tokio::time::timeout(
        DEPENDENCY_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            let mut conn = pool.get().map_err(|e| format!("pool: {e}"))?;
            diesel::sql_query("SELECT 1")
                .execute(&mut conn)
                .map_err(|e| format!("query: {e}"))?;
            Ok::<(), String>(())
        }),
    )
    .await;
    let ok = match outcome {
        Ok(Ok(Ok(()))) => true,
        Ok(Ok(Err(e))) => {
            log::error!("[ready] postgres check failed: {e}");
            false
        }
        Ok(Err(e)) => {
            log::error!("[ready] postgres probe task failed: {e}");
            false
        }
        Err(_) => {
            log::error!(
                "[ready] postgres probe timed out after {}ms",
                DEPENDENCY_TIMEOUT.as_millis()
            );
            false
        }
    };
    from_outcome(ok, started)
}

#[cfg(feature = "cache")]
async fn check_cache(client: &redis::Client) -> DependencyProbe {
    let client = client.clone();
    let started = Instant::now();
    let outcome = tokio::time::timeout(
        DEPENDENCY_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            let mut conn = client
                .get_connection_with_timeout(DEPENDENCY_TIMEOUT)
                .map_err(|e| format!("connect: {e}"))?;
            let reply = redis::cmd("PING")
                .query::<String>(&mut conn)
                .map_err(|e| format!("ping: {e}"))?;
            Ok::<bool, String>(reply.eq_ignore_ascii_case("PONG"))
        }),
    )
    .await;
    let ok = match outcome {
        Ok(Ok(Ok(value))) => value,
        Ok(Ok(Err(e))) => {
            log::error!("[ready] cache check failed: {e}");
            false
        }
        Ok(Err(e)) => {
            log::error!("[ready] cache probe task failed: {e}");
            false
        }
        Err(_) => {
            log::error!(
                "[ready] cache probe timed out after {}ms",
                DEPENDENCY_TIMEOUT.as_millis()
            );
            false
        }
    };
    from_outcome(ok, started)
}

async fn check_drive(
    drive: &Arc<dyn botlib::traits::DriveRepository>,
    bucket: &str,
) -> DependencyProbe {
    let drive = drive.clone();
    let bucket = bucket.to_string();
    let started = Instant::now();
    let outcome = tokio::time::timeout(DEPENDENCY_TIMEOUT, drive.head_bucket(&bucket)).await;
    let ok = match outcome {
        Ok(Ok(value)) => value,
        Ok(Err(e)) => {
            log::error!("[ready] drive check failed: {e}");
            false
        }
        Err(_) => {
            log::error!(
                "[ready] drive probe timed out after {}ms",
                DEPENDENCY_TIMEOUT.as_millis()
            );
            false
        }
    };
    from_outcome(ok, started)
}

/// Extracts host and port from an LLM endpoint URL without depending on the
/// feature-gated `monitoring` module.
#[cfg(feature = "llm")]
fn llm_host_port(url: &str, default_port: u16) -> Option<(String, u16)> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit('@').next()?;
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().ok()?),
        None => (authority, default_port),
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() {
        return None;
    }
    Some((host.to_string(), port))
}

#[cfg(feature = "llm")]
async fn check_llm(state: &Arc<AppState>) -> DependencyProbe {
    let started = Instant::now();
    if state.llm_provider.is_none() {
        return DependencyProbe::not_configured();
    }
    // Vault-backed LLM settings are not exposed as a client on AppState; when
    // no endpoint URL is published via env we can only report "configured".
    let Ok(url) = std::env::var("LLM_URL").or_else(|_| std::env::var("OLLAMA_HOST")) else {
        return DependencyProbe::ok(started);
    };
    let Some((host, port)) = llm_host_port(&url, 8081) else {
        return DependencyProbe::ok(started);
    };
    let outcome = tokio::time::timeout(
        DEPENDENCY_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            use std::net::{TcpStream, ToSocketAddrs};
            let Ok(addresses) = (host.as_str(), port).to_socket_addrs() else {
                return false;
            };
            for address in addresses {
                if TcpStream::connect_timeout(&address, DEPENDENCY_TIMEOUT).is_ok() {
                    return true;
                }
            }
            false
        }),
    )
    .await;
    let ok = match outcome {
        Ok(Ok(value)) => value,
        Ok(Err(e)) => {
            log::error!("[ready] llm probe task failed: {e}");
            false
        }
        Err(_) => {
            log::error!(
                "[ready] llm probe timed out after {}ms",
                DEPENDENCY_TIMEOUT.as_millis()
            );
            false
        }
    };
    from_outcome(ok, started)
}

async fn readiness_response(state: &Arc<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    // All probes are independent: start them together and wait once.
    let postgres_future = check_postgres(state);
    let cache_future = async {
        #[cfg(feature = "cache")]
        {
            match state.cache.as_ref() {
                Some(client) => check_cache(client).await,
                None => DependencyProbe::not_configured(),
            }
        }
        #[cfg(not(feature = "cache"))]
        {
            DependencyProbe::not_configured()
        }
    };
    let drive_future = async {
        match state.drive.as_ref() {
            Some(drive) => check_drive(drive, &state.bucket_name).await,
            None => DependencyProbe::not_configured(),
        }
    };
    let llm_future = async {
        #[cfg(feature = "llm")]
        {
            check_llm(state).await
        }
        #[cfg(not(feature = "llm"))]
        {
            DependencyProbe::not_configured()
        }
    };
    let (postgres, cache, drive, llm) =
        tokio::join!(postgres_future, cache_future, drive_future, llm_future);

    // No Vault client is held on AppState. Report it honestly rather than
    // fabricating one (issue #2385).
    let vault = DependencyProbe::not_configured();

    let mut failing = Vec::new();
    for (name, probe) in [
        ("postgres", &postgres),
        ("cache", &cache),
        ("drive", &drive),
        ("llm", &llm),
        ("vault", &vault),
    ] {
        if probe.is_failed() {
            failing.push(name);
        }
    }

    // PostgreSQL is the only critical dependency: an unavailable cache or LLM
    // degrades the service but must not make readiness fail (desktop-local
    // builds run without them).
    let unavailable = postgres.is_failed();
    let status = if unavailable {
        "unavailable"
    } else if failing.is_empty() {
        "ok"
    } else {
        "degraded"
    };
    let code = if unavailable {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    let build_date = botcore::product::build_info::build_date();
    let commit = botcore::product::build_info::commit();
    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "botserver",
            "version": botcore::product::build_info::version(),
            "build_date": build_date,
            "commit": commit,
            "failing": failing,
            "dependencies": {
                "postgres": postgres,
                "cache": cache,
                "drive": drive,
                "llm": llm,
                "vault": vault
            }
        })),
    )
}

/// GET /ready — dependency-aware readiness for load balancers and the sidecar.
pub async fn health_ready(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    readiness_response(&state).await
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let pool = state.conn.clone();
    let db_ok = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::task::spawn_blocking(move || pool.get().is_ok()),
    )
    .await
    .unwrap_or(Ok(false))
    .unwrap_or(false);

    let status = if db_ok { "healthy" } else { "degraded" };
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    // #2103 — one build metadata source for every endpoint.
    let build_date = botcore::product::build_info::build_date();
    let commit = botcore::product::build_info::commit();

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "botserver",
            "version": botcore::product::build_info::version(),
            "build_date": build_date,
            "commit": commit,
            "database": db_ok
        })),
    )
}

#[derive(serde::Deserialize)]
struct HealthQuery {
    deep: Option<u8>,
}

/// GET /health is a fast liveness stub; `?deep=1` opts into the same probes as
/// /ready for callers that cannot use a separate path.
pub async fn health_check_simple(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HealthQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    if query.deep == Some(1) {
        return readiness_response(&state).await;
    }
    let commit = botcore::product::build_info::commit();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "botserver",
            "version": botcore::product::build_info::version(),
            "commit": commit
        })),
    )
}

#[derive(serde::Deserialize)]
pub struct ClientErrorsRequest {
    errors: Vec<ClientErrorData>,
}

#[derive(serde::Deserialize)]
pub struct ClientErrorData {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    stack: Option<String>,
    #[serde(default)]
    url: String,
    #[serde(default)]
    timestamp: String,
}

pub async fn receive_client_errors(
    Json(payload): Json<ClientErrorsRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    for error in &payload.errors {
        log::error!(
            "[CLIENT ERROR] {} | {} | {} | URL: {} | Stack: {}",
            error.timestamp,
            error.r#type,
            error.message,
            error.url,
            error.stack.as_deref().unwrap_or("<no stack>")
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "count": payload.errors.len()
        })),
    )
}
