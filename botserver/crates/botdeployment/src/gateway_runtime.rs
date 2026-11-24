//! App runtime bootstrap for deployed Incus containers.
//!
//! After the deploy gateway pushes an app artifact into `/opt/app/` and
//! starts the container, nothing used to run the application itself — `ps`
//! showed no node/python/binary process and the app was unreachable. This
//! module auto-detects the runtime (node, python, rust binary, or static
//! files), installs the runtime when missing, and manages the app with a
//! systemd unit (`app.service`) bound to port 80 inside the container, so
//! the app is always running, auto-restarts on crash, and starts on boot.

use std::time::Duration;

use base64::Engine;
use botlib::security::SafeCommand;

use super::gateway_server::GatewayState;

/// Strict allowlist for a per-deployment PostgreSQL URL (#2652).
///
/// The gateway handler validates the same value, but the script boundary must
/// fail closed on its own: a caller that bypasses the HTTP handler must never
/// get a hostile value encoded into a root-written systemd unit (a decoded
/// `\nExecStartPre=…` would run as root when the service starts). Error
/// messages stay generic so no credential material is logged.
fn validate_database_url(url: &str) -> Result<(), String> {
    if url.is_empty()
        || url.len() > 512
        || (!url.starts_with("postgres://") && !url.starts_with("postgresql://"))
        || url.contains("UNIT")
        || url.chars().any(|c| {
            !(c.is_ascii_alphanumeric()
                || matches!(
                    c,
                    ':' | '/' | '@' | '?' | '=' | '.' | '_' | '+' | '~' | '%' | '-'
                ))
        })
    {
        return Err("invalid database_url: rejected at script boundary".to_string());
    }
    Ok(())
}

/// Quote a shell word safely: wrap in single quotes and escape each embedded
/// `'` as `'\''`, so the value cannot break out of its argument (#2652).
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Bootstrap script executed inside the container via `incus exec bash -c`.
///
/// #1386/#2652 — the caller-supplied URL is never written verbatim into the
/// heredoc. It is validated first (fail closed), then base64-encoded (a strict
/// `[A-Za-z0-9+/=]` alphabet) and decoded by the shell while the unit file is
/// created, so a newline or heredoc terminator in the request cannot add
/// commands or systemd directives to the root shell script/unit. The encoded
/// payload is additionally single-quoted with embedded-quote escaping.
fn bootstrap_script(database_url: Option<&str>) -> Result<String, String> {
    let db_env_line = match database_url {
        None => String::new(),
        Some(url) => {
            validate_database_url(url)?;
            let encoded = base64::engine::general_purpose::STANDARD.encode(url.as_bytes());
            format!(
                "Environment=DATABASE_URL=$(printf '%s' {} | base64 -d)",
                shell_single_quote(&encoded)
            )
        }
    };
    Ok(BOOTSTRAP_SCRIPT.replace("{DB_ENV_LINE}", &db_env_line))
}

#[cfg(test)]
mod db_env_tests {
    use super::{bootstrap_script, shell_single_quote, validate_database_url};

    #[test]
    fn url_never_appears_verbatim_in_generated_script() {
        let url = "postgresql://gb:secret@db.example:5432/app";
        let script = bootstrap_script(Some(url)).expect("legitimate URL");
        assert!(
            !script.contains(url),
            "raw URL must not reach the shell script"
        );
        assert!(script.contains("base64 -d"));
        assert!(!bootstrap_script(None)
            .expect("no URL")
            .contains("{DB_ENV_LINE}"));
    }

    #[test]
    fn hostile_urls_fail_closed() {
        for url in [
            "postgresql://u:p@db:5432/app\nUNIT\nid",
            "postgresql://u:p@db:5432/app\r\nExecStartPre=/bin/sh -c 'id'",
            "postgresql://u:p@db:5432/app\0id",
            "postgresql://u:p@db:5432/app$HOME",
            "postgresql://u:p@db:5432/app`id`",
            "postgresql://u:p@db:5432/app;id",
            "postgresql://u:p@db:5432/app&id",
            "postgresql://u:p@db:5432/app|id",
            "postgresql://u:p@db:5432/app'quoted'",
            "postgresql://u:p@db:5432/app with space",
            "mysql://db/app",
        ] {
            assert!(
                validate_database_url(url).is_err(),
                "hostile URL must be rejected: {url:?}"
            );
            assert!(
                bootstrap_script(Some(url)).is_err(),
                "hostile URL must never produce a script: {url:?}"
            );
        }
    }

    #[test]
    fn shell_single_quote_escapes_embedded_quotes() {
        assert_eq!(shell_single_quote("plain"), "'plain'");
        assert_eq!(shell_single_quote("a'b"), "'a'\\''b'");
        assert_eq!(shell_single_quote(""), "''");
    }
}

const BOOTSTRAP_SCRIPT: &str = r#"set -e
cd /opt/app || { echo "no /opt/app directory"; exit 1; }
for f in *.tar.gz *.tgz; do [ -f "$f" ] || continue; tar -xzf "$f"; rm -f "$f"; done
RUNTIME=static
if [ -f package.json ]; then
  RUNTIME=node
  command -v node >/dev/null 2>&1 || { apt-get update -qq && apt-get install -y -qq nodejs npm; }
  [ -d node_modules ] || npm install --no-audit --no-fund --silent
  START=""
  if node -e "const p=require('./package.json');process.exit(p.scripts&&p.scripts.start?0:1)" 2>/dev/null; then START="npm start"; fi
  if [ -z "$START" ] && [ -f server.js ]; then START="node server.js"; fi
  if [ -z "$START" ] && [ -f index.js ]; then START="node index.js"; fi
  if [ -z "$START" ] && [ -f app.js ]; then START="node app.js"; fi
  [ -z "$START" ] && START="node index.js"
elif [ -f requirements.txt ] || [ -f pyproject.toml ] || [ -f app.py ] || [ -f main.py ] || [ -f server.py ] || [ -f manage.py ]; then
  RUNTIME=python
  command -v python3 >/dev/null 2>&1 || { apt-get update -qq && apt-get install -y -qq python3 python3-pip; }
  [ -f requirements.txt ] && pip3 install -q -r requirements.txt || true
  if [ -f app.py ]; then START="python3 app.py";
  elif [ -f main.py ]; then START="python3 main.py";
  elif [ -f server.py ]; then START="python3 server.py";
  elif [ -f manage.py ]; then START="python3 manage.py runserver 0.0.0.0:80";
  else START="python3 app.py"; fi
elif [ -f Cargo.toml ] || ls target/release/* >/dev/null 2>&1; then
  RUNTIME=rust
  BIN=""
  if ls target/release/* >/dev/null 2>&1; then
    BIN=$(find target/release -maxdepth 1 -type f -executable ! -name '*.d' -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)
  fi
  if [ -z "$BIN" ] && [ -f Cargo.toml ]; then
    command -v cargo >/dev/null 2>&1 || { apt-get update -qq && apt-get install -y -qq cargo; }
    cargo build --release --quiet
    BIN=$(find target/release -maxdepth 1 -type f -executable ! -name '*.d' -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)
  fi
  START="$BIN"
else
  BIN=$(find . -maxdepth 2 -type f -executable ! -name '*.sh' 2>/dev/null | head -1)
  if [ -n "$BIN" ]; then RUNTIME=rust; START="$BIN"; fi
fi
if [ "$RUNTIME" = "static" ]; then
  command -v python3 >/dev/null 2>&1 || { apt-get update -qq && apt-get install -y -qq python3; }
  START="python3 -m http.server 80 --bind 0.0.0.0"
fi
cat > /etc/systemd/system/app.service <<UNIT
[Unit]
Description=GB deployed app ($RUNTIME)
After=network.target

[Service]
WorkingDirectory=/opt/app
ExecStart=$START
Restart=always
RestartSec=3
Environment=PORT=80
{DB_ENV_LINE}
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable app >/dev/null 2>&1 || true
systemctl restart app
for i in $(seq 1 30); do
  if (exec 3<>/dev/tcp/127.0.0.1/80) 2>/dev/null; then exec 3>&- 3<&-; echo "APP_READY on :80 ($RUNTIME: $START)"; exit 0; fi
  sleep 1
done
echo "WARN: app not answering on :80 yet - check with: journalctl -u app -n 50"
exit 0
"#;

/// Run a shell script inside the container via `incus exec`.
///
/// Retries briefly when the container was just started and exec is not yet
/// available. The script is passed as a single argument to `bash -c`, so no
/// quoting of the caller's content is required.
async fn run_incus_exec(
    state: &GatewayState,
    container_name: &str,
    script: &str,
) -> Result<String, String> {
    // #1297 — the container name originates from deploy requests; restrict it
    // to a safe charset before it reaches the incus argv.
    if container_name.is_empty()
        || !container_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        || container_name.starts_with('-')
    {
        return Err(format!("invalid container name: {container_name}"));
    }
    let command = SafeCommand::new("incus")
        .and_then(|c| c.args(&["exec", container_name, "--", "bash", "-c"]))
        .and_then(|c| c.trusted_shell_script_arg(script))
        .and_then(|c| c.env("INCUS_SOCKET", &state.incus_socket))
        .map_err(|e| format!("Failed to build incus exec: {e}"))?;

    for attempt in 0..5 {
        let output = command
            .execute_async()
            .await
            .map_err(|e| format!("Failed to execute incus exec: {e}"))?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if attempt < 4 && stderr.contains("not running") {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        }
        return Err(format!("incus exec {container_name}: {stderr}"));
    }
    Err(format!("incus exec {container_name}: container not ready"))
}

/// Detect the app runtime inside the container and start it as a managed
/// systemd service bound to port 80.
///
/// The container must already be running with the artifact at `/opt/app/`.
/// Returns the bootstrap output (runtime detected + readiness) for logging.
pub async fn bootstrap_app_runtime(
    state: &GatewayState,
    container_name: &str,
    database_url: Option<&str>,
) -> Result<String, String> {
    let script = bootstrap_script(database_url)?;
    let output = run_incus_exec(state, container_name, &script).await?;
    log::info!("Runtime bootstrap ({container_name}): {}", output.trim());
    Ok(output)
}
