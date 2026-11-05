//! Workspace side of conversation autotasks (#1613).
//!
//! Every job is a shell script under
//! `/opt/vibe/conversations/{conversation}/autotasks/{id}.sh` plus either a
//! crontab entry (`kind=cron`) or a systemd service (`kind=systemd`) inside the
//! user's agent container. The database row mirrors what this module observes;
//! it never assumes an entry exists because the row says so.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use botcore::shared::state::AppState;
use botschema::conversation_autotasks as table;

use super::{AutotaskRow, STATUS_ORPHANED, STATUS_PENDING, STATUS_RUNNING, STATUS_SCHEDULED};
use crate::core::bot::agent_vm_hook;

pub(super) const CONVERSATIONS_ROOT: &str = "/opt/vibe/conversations";

pub(super) fn conversation_root(conversation_id: Uuid) -> String {
    format!("{CONVERSATIONS_ROOT}/{conversation_id}")
}

pub(super) fn unit_name_for(id: Uuid) -> String {
    format!("gb-autotask-{id}")
}

/// Writes the job body inside the workspace and installs the schedule.
pub(super) async fn install(
    user_id: Uuid,
    bot_id: Uuid,
    conversation_id: Uuid,
    id: Uuid,
    kind: &str,
    schedule: &str,
    command: &str,
    unit: &str,
) -> Result<String, String> {
    let root = conversation_root(conversation_id);
    let encoded = BASE64.encode(command.as_bytes());
    let script = format!(
        "set -e\nmkdir -p {root}/autotasks\n\
         printf %s '{encoded}' | base64 -d > {root}/autotasks/{id}.sh\n\
         chmod +x {root}/autotasks/{id}.sh\n"
    );
    let install = if kind == "systemd" {
        format!(
            "{script}\
             cat > /etc/systemd/system/{unit}.service <<'GBUNIT'\n\
             [Unit]\nDescription=General Bots conversation autotask {id}\n\
             [Service]\nType=simple\nRestart=on-failure\n\
             ExecStart=/bin/sh {root}/autotasks/{id}.sh\n\
             [Install]\nWantedBy=multi-user.target\nGBUNIT\n\
             systemctl daemon-reload\nsystemctl enable --now {unit}.service\n\
             systemctl is-active {unit}.service\n"
        )
    } else {
        format!(
            "{script}\
             command -v crontab >/dev/null 2>&1 || \\
               (apt-get -o DPkg::Lock::Timeout=300 update -q && \\
                DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=300 install -y -q cron) >/dev/null 2>&1 || true\n\
             (crontab -l 2>/dev/null | grep -v 'gb-autotask:{id}' || true; \
             echo '{schedule} /bin/sh {root}/autotasks/{id}.sh # gb-autotask:{id}') | crontab -\n\
             crontab -l | grep -c 'gb-autotask:{id}'\n"
        )
    };
    // Installers need root (systemd units, crontab); generated programs still
    // run as `runner` through run_shell_in_conversation.
    match agent_vm_hook::run_shell_in_conversation_as_root(
        user_id,
        bot_id,
        conversation_id,
        &install,
        120_000,
    )
    .await
    {
        Some((true, stdout, _stderr)) => {
            if kind == "systemd" && stdout.trim() == "active" {
                Ok(STATUS_RUNNING.to_string())
            } else if kind != "systemd" && stdout.trim() != "0" {
                Ok(STATUS_SCHEDULED.to_string())
            } else {
                Ok(STATUS_PENDING.to_string())
            }
        }
        Some((false, _out, stderr)) => Err(stderr),
        None => Err("the agent workspace is not available".to_string()),
    }
}

/// Asks the workspace whether the job is still there and updates the row.
///
/// The database connection is acquired only for the update: the workspace call
/// can take seconds and must never pin a pooled connection.
pub(super) async fn refresh(state: &AppState, row: &AutotaskRow) {
    let unit = row.unit_name.clone().unwrap_or_else(|| unit_name_for(row.id));
    let check = if row.kind == "systemd" {
        format!("systemctl is-active {unit}.service 2>/dev/null || true")
    } else {
        format!(
            "crontab -l 2>/dev/null | grep -c 'gb-autotask:{}' || true",
            row.id
        )
    };
    // Same identity as the installer: root owns the crontab/systemd unit.
    // Checking as `runner` made every installed job look orphaned.
    let observed = agent_vm_hook::run_shell_in_conversation_as_root(
        row.user_id,
        row.bot_id,
        row.conversation_id,
        &check,
        15_000,
    )
    .await;
    let Some((ok, stdout, _stderr)) = observed else {
        return;
    };
    let output = stdout.trim();
    let active = ok
        && if row.kind == "systemd" {
            output == "active"
        } else {
            output != "0" && !output.is_empty()
        };
    let next = if active {
        if row.kind == "systemd" {
            STATUS_RUNNING
        } else {
            STATUS_SCHEDULED
        }
    } else {
        STATUS_ORPHANED
    };
    if next == row.status {
        return;
    }
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::warn!("autotask reconciliation skipped: {error}");
            return;
        }
    };
    let _ = diesel::update(table::table.find(row.id))
        .set((table::status.eq(next), table::updated_at.eq(Utc::now())))
        .execute(&mut conn);
}

/// Removes the schedule and the script from the workspace.
pub(super) async fn remove(row: &AutotaskRow) -> Option<String> {
    let unit = row.unit_name.clone().unwrap_or_else(|| unit_name_for(row.id));
    let root = conversation_root(row.conversation_id);
    let id = row.id;
    let removal = if row.kind == "systemd" {
        format!(
            "systemctl disable --now {unit}.service 2>/dev/null || true\n\
             rm -f /etc/systemd/system/{unit}.service\nsystemctl daemon-reload\n\
             rm -f {root}/autotasks/{id}.sh\n"
        )
    } else {
        format!(
            "(crontab -l 2>/dev/null | grep -v 'gb-autotask:{id}' || true) | crontab -\n\
             rm -f {root}/autotasks/{id}.sh\n"
        )
    };
    match agent_vm_hook::run_shell_in_conversation_as_root(
        row.user_id,
        row.bot_id,
        row.conversation_id,
        &removal,
        30_000,
    )
    .await
    {
        Some((true, _stdout, _stderr)) => None,
        Some((false, _stdout, stderr)) => Some(stderr),
        None => Some("the agent workspace is not available".to_string()),
    }
}
