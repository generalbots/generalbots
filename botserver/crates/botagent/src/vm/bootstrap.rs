//! Container bootstrap and idempotent workspace preparation scripts.

use std::sync::Arc;

use crate::AgentService;

use super::incus::exec_in_vm;

/// Installs the runtime and enforces internet-only egress inside the
/// container. It runs once at provision and appends to
/// `/tmp/agent-bootstrap.log`. Private ranges are rejected while the bridge
/// resolver keeps working, so package installs and public APIs are reachable
/// but the host, the LAN and sibling tenants are not.
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
  # Base data/document toolkit: the model should not pay an install round-trip
  # for the common agent outputs (decks, sheets, charts, PDFs).
  python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
    pandas numpy openpyxl python-pptx pillow matplotlib requests python-docx \
    reportlab beautifulsoup4 lxml tabulate || true
  echo v4 > /opt/vibe/.provisioned
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

/// Bumped whenever the base toolkit changes; existing VMs run the prepare
/// step once when their marker does not match.
/// #2062 — image profile. `office` is the default: document libraries plus a
/// headless Chromium used only for HTML->PDF and screenshots. Media/ML stacks
/// stay in BotModels; the bottle profile never bakes torch/ffmpeg/libreoffice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VmProfile {
    Base,
    Office,
    Media,
    Data,
}

impl VmProfile {
    pub(super) fn from_env() -> Self {
        match std::env::var("GB_VM_PROFILE")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "base" => Self::Base,
            "media" => Self::Media,
            "data" => Self::Data,
            _ => Self::Office,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Office => "office",
            Self::Media => "media",
            Self::Data => "data",
        }
    }

    /// Extra shell install steps. Every command is best-effort (`|| true`): a
    /// profile that cannot install yet must not fail provisioning.
    pub(super) fn install_block(self) -> &'static str {
        match self {
            Self::Base => "",
            Self::Office => r#"
# #2062 office profile: headless Chromium (print-to-pdf/screenshots) and
# fonts, with the document libraries already installed above. No LibreOffice
# cold start, no ffmpeg, no torch.
apt-get -o DPkg::Lock::Timeout=300 install -y -q chromium fonts-noto-core fonts-dejavu-core || true
python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
  pikepdf pypdf markdown jinja2 tabulate || true
"#,
            Self::Media => r#"
# #2062 media profile (opt-in): local transcode only. Generation, music,
# video and AI stay in BotModels; no CUDA/torch stack is installed.
apt-get -o DPkg::Lock::Timeout=300 install -y -q ffmpeg || true
"#,
            Self::Data => r#"
# #2062 data profile (opt-in): scientific python for notebooks and joins.
python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
  scipy statsmodels || true
"#,
        }
    }

    /// Marker written to `/opt/vibe/.provisioned`. A profile change re-runs the
    /// preparation exactly once.
    pub(super) fn prepare_version(self) -> String {
        format!("v6-{}", self.as_str())
    }
}

/// The bootstrap script for the active profile, including the version marker.
pub(super) fn bootstrap_script() -> String {
    let profile = VmProfile::from_env();
    format!(
        "{BOOTSTRAP_SCRIPT}\n{}\necho {} > /opt/vibe/.provisioned\n",
        profile.install_block(),
        profile.prepare_version()
    )
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
  python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
    pandas numpy openpyxl python-pptx pillow matplotlib requests python-docx \
    reportlab beautifulsoup4 lxml tabulate || true
} >/tmp/agent-prepare.log 2>&1 || true
"#;

/// Preparation script for the active profile (base toolkit + profile packages
/// + the version marker that makes the run idempotent).
pub(super) fn workspace_prepare_script() -> String {
    let profile = VmProfile::from_env();
    format!(
        "{WORKSPACE_PREPARE_SCRIPT_BASE}\n{}\necho {} > /opt/vibe/.provisioned\n",
        profile.install_block(),
        profile.prepare_version()
    )
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
                    version = VmProfile::from_env().prepare_version()
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

#[cfg(test)]
mod profile_tests {
    use super::*;

    #[test]
    fn the_office_profile_keeps_the_image_small() {
        let block = VmProfile::Office.install_block();
        assert!(block.contains("chromium"));
        assert!(block.contains("fonts-noto-core"));
        // #2062 — media/ML and LibreOffice stay out of the default image.
        for forbidden in ["ffmpeg", "libreoffice", "torch", "tensorflow", "cuda"] {
            assert!(
                !block.to_ascii_lowercase().contains(forbidden),
                "office profile must not install {forbidden}"
            );
        }
    }

    #[test]
    fn media_and_data_are_opt_in_profiles() {
        assert!(VmProfile::Media.install_block().contains("ffmpeg"));
        assert!(VmProfile::Data.install_block().contains("scipy"));
        assert!(VmProfile::Base.install_block().is_empty());
    }

    #[test]
    fn the_prepare_version_names_the_profile_so_a_change_reprovisions() {
        assert_ne!(
            VmProfile::Office.prepare_version(),
            VmProfile::Base.prepare_version()
        );
        assert!(VmProfile::Office.prepare_version().contains("office"));
        assert!(VmProfile::Base.prepare_version().contains("base"));
    }

    #[test]
    fn an_unknown_profile_falls_back_to_office() {
        // The parser is pure on the value it receives from the environment; a
        // typo must not leave a VM without document support.
        assert_eq!(VmProfile::from_env(), VmProfile::Office);
    }
}
