//! `vm_incus::isolation_host` — #2653 Incus CLI provisioning for the Vibe
//! tenant-isolation ACL. Split from `linux.rs` to keep that module small; the
//! pure allow/drop decisions live in `isolation.rs`.

use super::*;
use super::isolation;

/// True when `show` output already contains the value a rule matches on.
/// Incus renders rules in YAML (`source: 10.123.45.2/31`), not in the CLI's
/// `key=value` form, so object identity is checked by the matched CIDR.
fn acl_rule_present(show: &str, properties: &str) -> bool {
    let target = match properties.split_once(',') {
        Some((_, pair)) => match pair.split_once('=') {
            Some((_, value)) => value,
            None => pair,
        },
        None => properties,
    };
    show.contains(target)
}

impl VmLifecycle {
    /// Apply the isolation ACL to an existing VM's `eth0` before it starts.
    /// Covers containers created before #2653 and any NIC whose ACL was lost
    /// when the bridge subnet changed. Fail closed: a VM never starts with
    /// an unisolated bridge NIC.
    pub(crate) fn ensure_nic_isolation(&self, name: &str) -> Result<(), String> {
        let bridge = self.vibe_bridge()?;
        let acl = self.ensure_isolation_acl(&bridge)?;
        self.incus_run(
            &[
                "config".to_string(),
                "device".to_string(),
                "set".to_string(),
                name.to_string(),
                "eth0".to_string(),
                format!("security.acls={acl}"),
            ],
            30,
        )
        .map_err(|e| format!("apply isolation ACL {acl} to {name}/eth0: {e}"))?;
        Ok(())
    }

    /// Resolve the managed Incus bridge Vibe VMs attach to. Prefer the
    /// default bridge (`incusbr0`), then any managed bridge; never a physical
    /// NIC. Failure to find one is an error, not a fallback to a hard-coded
    /// name: creating an unisolated VM is worse than refusing to create one.
    pub(crate) fn vibe_bridge(&self) -> Result<String, String> {
        let out = self
            .incus_run(
                &[
                    "network".to_string(),
                    "list".to_string(),
                    "--format".to_string(),
                    "csv".to_string(),
                ],
                30,
            )
            .map_err(|e| format!("list Incus networks for Vibe isolation: {e}"))?;
        // CSV columns: name,type,managed,ipv4,ipv6,description,usedby,state
        let rows: Vec<Vec<String>> = out
            .stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.split(',')
                    .map(|col| col.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .collect();
        let bridge = rows
            .iter()
            .find(|r| r.first().map(String::as_str) == Some("incusbr0"))
            .or_else(|| {
                rows.iter()
                    .find(|r| r.get(1).map(String::as_str) == Some("bridge"))
            })
            .and_then(|r| r.first().cloned())
            .filter(|name| !name.is_empty());
        match bridge {
            Some(name) => Ok(name),
            None => Err(
                "no managed Incus bridge found; refusing to create an unisolated Vibe VM"
                    .to_string(),
            ),
        }
    }

    /// Create and fully configure the ACL that denies VM-to-VM traffic on
    /// the shared bridge, returning its name so the caller can attach ONLY
    /// that isolated NIC. Rules are derived from the bridge's live subnet,
    /// so a changed subnet produces a new ACL name instead of a stale rule
    /// set. Any failure propagates: `linux_create` aborts before the
    /// container is launched, which is the fail-closed behaviour #2653 needs.
    pub(crate) fn ensure_isolation_acl(&self, bridge: &str) -> Result<String, String> {
        let addr = self
            .incus_run(
                &[
                    "network".to_string(),
                    "get".to_string(),
                    bridge.to_string(),
                    "ipv4.address".to_string(),
                ],
                30,
            )
            .map_err(|e| format!("read bridge {bridge} ipv4.address for isolation: {e}"))?;
        let (gateway_ip, subnet_cidr) = isolation::bridge_subnet(addr.stdout.trim())?;
        // Prove the policy before touching Incus: the gateway must stay
        // reachable and a synthetic tenant address must be denied.
        let probe_ip = isolation::bridge_probe_ip(&subnet_cidr, &gateway_ip)?;
        let gateway_decision =
            isolation::destination_allowed(&gateway_ip, &gateway_ip, &subnet_cidr)?;
        let probe_decision = isolation::destination_allowed(&probe_ip, &gateway_ip, &subnet_cidr)?;
        if gateway_decision != isolation::IsolationDecision::Allow
            || probe_decision != isolation::IsolationDecision::Drop
        {
            return Err("Vibe isolation policy self-check failed; refusing to create an unisolated VM".to_string());
        }

        let acl = isolation::acl_name(&subnet_cidr)?;
        let rules = isolation::isolation_rules(&gateway_ip, &subnet_cidr)?;
        let show = |acl: &str| -> Result<String, String> {
            self.incus_run(
                &[
                    "network".to_string(),
                    "acl".to_string(),
                    "show".to_string(),
                    acl.to_string(),
                ],
                30,
            )
            .map(|out| out.stdout)
            .map_err(|e| format!("show isolation ACL {acl}: {e}"))
        };

        // Idempotent create: a concurrent VM may have created it already.
        if show(&acl).is_err() {
            self.incus_run(
                &[
                    "network".to_string(),
                    "acl".to_string(),
                    "create".to_string(),
                    acl.clone(),
                ],
                30,
            )
            .map_err(|e| format!("create isolation ACL {acl}: {e}"))?;
        }
        // Re-read after create (or after the concurrent-create race) so the
        // rule checks below always start from real ACL state.
        let mut current = show(&acl)?;

        // A crash between create and rule-add can leave a partial ACL. Add
        // whatever is missing; duplicates are skipped by checking `show`.
        for rule in &rules {
            if acl_rule_present(&current, &rule.properties) {
                continue;
            }
            let mut args = vec![
                "network".to_string(),
                "acl".to_string(),
                "rule".to_string(),
                "add".to_string(),
                acl.clone(),
                rule.direction.to_string(),
            ];
            args.extend(rule.properties.split(',').map(str::to_string));
            self.incus_run(&args, 30)
                .map_err(|e| format!("add rule {} to isolation ACL {acl}: {e}", rule.properties))?;
            current = show(&acl)?;
        }

        // Final verification: every rule must really be present, otherwise
        // fail closed instead of attaching a NIC to a partially configured
        // ACL.
        let missing: Vec<&str> = rules
            .iter()
            .filter(|rule| !acl_rule_present(&current, &rule.properties))
            .map(|rule| rule.properties.as_str())
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "isolation ACL {acl} is incomplete ({} rule(s) missing); refusing to create VM",
                missing.len()
            ));
        }
        Ok(acl)
    }
}
