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

use super::gateway_server::GatewayState;

/// Bootstrap script executed inside the container via `incus exec bash -c`.
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
    let mut cmd = tokio::process::Command::new("incus");
    cmd.args(["exec", container_name, "--", "bash", "-c", script]);
    cmd.env("INCUS_SOCKET", &state.incus_socket);

    for attempt in 0..5 {
        let output = cmd
            .output()
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
) -> Result<String, String> {
    let output = run_incus_exec(state, container_name, BOOTSTRAP_SCRIPT).await?;
    log::info!(
        "Runtime bootstrap ({container_name}): {}",
        output.trim()
    );
    Ok(output)
}