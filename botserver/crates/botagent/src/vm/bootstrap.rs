//! Container bootstrap and idempotent workspace preparation scripts.
//!
//! Both scripts are `base runtime` (best effort: a slow mirror must not block
//! the workspace) plus the #2062 profile block (strict: a missing document
//! engine is a capability failure, never a silent downgrade). The profile data
//! and the generated install steps live in [`super::profile`].

use std::sync::Arc;

use crate::AgentService;

use super::incus::exec_in_vm;
use super::profile::{active_profile, install_block};

/// Installs the runtime and enforces internet-only egress inside the
/// container. It runs once at provision and appends to
/// `/tmp/agent-bootstrap.log`. Private ranges are rejected while the bridge
/// resolver keeps working, so package installs and public APIs are reachable
/// but the host, the LAN and sibling tenants are not.
///
/// Only the base runtime is installed here; the document/browser stack comes
/// from the active profile (`profile::install_block`) so a `base` VM stays as
/// small as today's.
pub(super) const BOOTSTRAP_SCRIPT: &str = r#"
set -eu
export DEBIAN_FRONTEND=noninteractive
# Debian containers can boot without /etc/resolv.conf; apt then fails to
# resolve and the runtime is never installed ("python3: not found" on the
# first vm_exec). Point the container at the bridge resolver before apt.
GW=$(ip route 2>/dev/null | awk '/^default/ {print $3; exit}')
{
  [ -n "${GW:-}" ] && echo "nameserver $GW"
  echo "nameserver 1.1.1.1"
  echo "nameserver 8.8.8.8"
} > /etc/resolv.conf
{
  apt-get -o DPkg::Lock::Timeout=300 update -q
  # `cron` powers conversation autotasks (scheduled shell jobs); an image
  # without it made every `kind=cron` install fail with "crontab: not found".
  apt-get -o DPkg::Lock::Timeout=300 install -y -q python3 python3-pip nodejs npm iptables iproute2 cron
  mkdir -p /opt/vibe/conversations
  # Generic helpers only: the document/PDF/browser stack belongs to the image
  # profile, so the default (`office`) and `base` stay separable.
  python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
    requests || true
  cat > /usr/local/sbin/agent-egress.sh <<'EOS'
#!/bin/sh
set -eu
GW=$(ip route | awk '/^default/ {print $3; exit}')
[ -n "$GW" ] || exit 0
iptables -F OUTPUT
iptables -A OUTPUT -o lo -j ACCEPT
iptables -A OUTPUT -d "$GW" -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -d "$GW" -p tcp --dport 53 -j ACCEPT
iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
for net in 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16 169.254.0.0/16; do
  iptables -A OUTPUT -d "$net" -j REJECT
done
iptables -A OUTPUT -j ACCEPT
EOS
  chmod +x /usr/local/sbin/agent-egress.sh
  cat > /etc/systemd/system/agent-egress.service <<'EOS'
[Unit]
Description=Agent internet-only egress
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/local/sbin/agent-egress.sh
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOS
  systemctl daemon-reload
  systemctl enable --now agent-egress.service
} >/tmp/agent-bootstrap.log 2>&1 || true
"#;

/// Appends the strict profile block and the marker that makes the run
/// idempotent. A profile failure aborts the script (exit 78) after recording
/// the reason in `/opt/vibe/.profile-error`, so the marker is never written for
/// a VM that cannot generate documents.
fn with_profile(base: &str) -> String {
    let profile = active_profile();
    format!(
        "{base}\n{}\necho {marker} > /opt/vibe/.provisioned\n",
        install_block(profile),
        marker = profile.prepare_version(),
    )
}

/// The bootstrap script for the active profile.
pub(super) fn bootstrap_script() -> String {
    with_profile(BOOTSTRAP_SCRIPT)
}

/// Idempotent preparation of an existing workspace VM: package index, base
/// Python toolkit and the conversations root. Runs in the background so the
/// first chat message is never blocked by apt/pip.
const WORKSPACE_PREPARE_SCRIPT_BASE: &str = r#"
set -eu
export DEBIAN_FRONTEND=noninteractive
# Debian containers can boot without /etc/resolv.conf; apt then fails to
# resolve and the runtime is never installed ("python3: not found" on the
# first vm_exec). Point the container at the bridge resolver before apt.
GW=$(ip route 2>/dev/null | awk '/^default/ {print $3; exit}')
{
  [ -n "${GW:-}" ] && echo "nameserver $GW"
  echo "nameserver 1.1.1.1"
  echo "nameserver 8.8.8.8"
} > /etc/resolv.conf
{
  mkdir -p /opt/vibe/conversations
  # #1658 — two identities: `claude` owns the subscription credentials,
  # `runner` executes model-generated code. Generated code must never be able
  # to read /home/claude.
  id -u runner >/dev/null 2>&1 || useradd -m -s /bin/bash runner
  id -u claude >/dev/null 2>&1 || useradd -m -s /bin/bash claude
  mkdir -p /home/claude/.claude
  chown -R claude:claude /home/claude
  chmod 700 /home/claude
  chmod 700 /home/claude/.claude
  chown -R runner:runner /opt/vibe/conversations
  command -v python3 >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q python3)
  python3 -m pip --version >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q python3-pip)
  command -v npm >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q nodejs npm)
  # #1652 — Claude Code subscription provider. The CLI runs as `claude`; its
  # credentials stay under /home/claude/.claude (0700). Installing it here
  # keeps the first "Connect Claude" click fast; a failed install is reported
  # by the provider, never silently treated as connected.
  command -v claude >/dev/null 2>&1 || npm install -g @anthropic-ai/claude-code >/tmp/claude-install.log 2>&1 || true
} >/tmp/agent-prepare.log 2>&1 || true
"#;

/// Preparation script for the active profile (base runtime + profile packages
/// + the version marker that makes the run idempotent).
pub(super) fn workspace_prepare_script() -> String {
    with_profile(WORKSPACE_PREPARE_SCRIPT_BASE)
}

/// Schedules workspace preparation once per VM per process. Never blocks the
/// caller: the on-demand installer covers a request that races the setup.
pub(super) fn schedule_workspace_preparation(state: Arc<AgentService>, vm_name: String) {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static IN_FLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let set = IN_FLIGHT.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut guard = match set.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if !guard.insert(vm_name.clone()) {
            return;
        }
    }
    tokio::spawn(async move {
        let ready = exec_in_vm(
            &state,
            &vm_name,
            &[
                "sh".into(),
                "-c".into(),
                format!(
                    "test -f /opt/vibe/.provisioned && grep -q '^{version}$' /opt/vibe/.provisioned",
                    version = active_profile().prepare_version()
                ),
            ],
        )
        .await;
        let already = matches!(ready, Ok((true, _, _)));
        if !already {
            let _ = exec_in_vm(
                &state,
                &vm_name,
                &["sh".into(), "-c".into(), workspace_prepare_script()],
            )
            .await;
        }
        if let Ok(mut guard) = set.lock() {
            guard.remove(&vm_name);
        }
    });
}
