//! `vm_incus::isolation` — #2653 tenant network isolation decisions.
//!
//! Every Vibe VM used to be plugged into the same managed Incus bridge with
//! no ACL, so code running in one tenant's VM could reach another tenant's
//! app port on that bridge. All host-side networking is expressed here as
//! pure functions so the allow/drop decision is unit-tested without Incus.
//!
//! Incus orders ACL rules by action (`drop`, `reject`, `allow`, default), not
//! by list order. A whole-subnet drop would therefore also drop the gateway
//! (host proxy / Caddy / DNS is inside that subnet). The drop rules below
//! cover the bridge subnet *minus the gateway*, decomposed into CIDR blocks
//! because IP-range subjects are not supported by every firewall driver.

use std::net::Ipv4Addr;

/// Prefix of the per-bridge isolation ACL name. The bridge CIDR is appended
/// so a changed subnet gets a fresh ACL instead of a stale rule set.
pub(crate) const ISOLATION_ACL_PREFIX: &str = "vibe-iso-";

/// Policy outcome for one packet crossing the isolation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IsolationDecision {
    /// Gateway (host/proxy) or off-bridge destination: permitted.
    Allow,
    /// Another VM on the same bridge: denied.
    Drop,
}

/// Contiguous IPv4 netmask for a prefix length (0..=32).
fn mask_for(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

/// Split a CIDR into its network base and prefix length, validating both.
fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, u8), String> {
    let (address, prefix) = cidr
        .split_once('/')
        .ok_or_else(|| format!("CIDR '{cidr}' has no prefix"))?;
    let network: Ipv4Addr = address
        .parse()
        .map_err(|e| format!("CIDR address '{address}' is invalid: {e}"))?;
    let prefix: u8 = prefix
        .parse()
        .map_err(|e| format!("CIDR prefix '{prefix}' is invalid: {e}"))?;
    if prefix > 32 {
        return Err(format!("CIDR prefix {prefix} is out of range"));
    }
    Ok((Ipv4Addr::from(u32::from(network) & mask_for(prefix)), prefix))
}

/// Derive the gateway IP and the full bridge subnet CIDR from the Incus
/// `ipv4.address` value (for example `10.123.45.1/24` → gateway
/// `10.123.45.1`, subnet `10.123.45.0/24`).
pub(crate) fn bridge_subnet(gateway_cidr: &str) -> Result<(String, String), String> {
    let (address, _) = gateway_cidr
        .split_once('/')
        .ok_or_else(|| format!("bridge ipv4.address '{gateway_cidr}' has no prefix"))?;
    let gateway: Ipv4Addr = address
        .parse()
        .map_err(|e| format!("bridge gateway '{address}' is invalid: {e}"))?;
    let (network, prefix) = parse_cidr(gateway_cidr)?;
    Ok((gateway.to_string(), format!("{network}/{prefix}")))
}

/// Decide whether `destination` may be reached from a Vibe VM. The gateway is
/// the host side of the bridge (proxy device / Caddy domain route) and must
/// stay reachable; anything else inside the bridge subnet is another tenant's
/// VM and is dropped. Off-bridge destinations (internet) are allowed.
pub(crate) fn destination_allowed(
    destination: &str,
    gateway_ip: &str,
    subnet_cidr: &str,
) -> Result<IsolationDecision, String> {
    let dest: Ipv4Addr = destination
        .parse()
        .map_err(|e| format!("destination '{destination}' is invalid: {e}"))?;
    let gateway: Ipv4Addr = gateway_ip
        .parse()
        .map_err(|e| format!("gateway '{gateway_ip}' is invalid: {e}"))?;
    if dest == gateway {
        return Ok(IsolationDecision::Allow);
    }
    let (network, prefix) = parse_cidr(subnet_cidr)?;
    let in_bridge = u32::from(dest) & mask_for(prefix) == u32::from(network);
    Ok(if in_bridge {
        IsolationDecision::Drop
    } else {
        IsolationDecision::Allow
    })
}

/// A deterministic address inside the bridge subnet that is not the gateway.
/// Provisioning uses it to prove that the policy denies VM-to-VM traffic
/// before any container is attached to the bridge.
pub(crate) fn bridge_probe_ip(subnet_cidr: &str, gateway_ip: &str) -> Result<String, String> {
    let (network, prefix) = parse_cidr(subnet_cidr)?;
    if prefix >= 31 {
        return Err(format!("bridge prefix /{prefix} has no usable probe host"));
    }
    let base = u32::from(network);
    let candidate = Ipv4Addr::from(base + 1);
    if candidate.to_string() == gateway_ip {
        return Ok(Ipv4Addr::from(base + 2).to_string());
    }
    Ok(candidate.to_string())
}

/// Recursively split `network/prefix` into CIDR blocks that cover every
/// address except `exclude`. Bounded to 32 blocks for a usable bridge subnet.
fn cidr_excluding(network: u32, prefix: u8, exclude: u32, out: &mut Vec<String>) {
    if prefix == 32 {
        if network != exclude {
            out.push(format!("{}/32", Ipv4Addr::from(network)));
        }
        return;
    }
    let half = 1u32 << (31 - prefix);
    let second = network + half;
    if exclude < second {
        out.push(format!("{}/{}", Ipv4Addr::from(second), prefix + 1));
        cidr_excluding(network, prefix + 1, exclude, out);
    } else {
        out.push(format!("{}/{}", Ipv4Addr::from(network), prefix + 1));
        cidr_excluding(second, prefix + 1, exclude, out);
    }
}

/// All CIDR blocks of the bridge subnet except the gateway address.
pub(crate) fn tenant_blocks(subnet_cidr: &str, gateway_ip: &str) -> Result<Vec<String>, String> {
    let (network, prefix) = parse_cidr(subnet_cidr)?;
    if prefix >= 31 {
        return Err(format!("bridge prefix /{prefix} cannot isolate tenant VMs"));
    }
    let gateway: Ipv4Addr = gateway_ip
        .parse()
        .map_err(|e| format!("gateway '{gateway_ip}' is invalid: {e}"))?;
    let gateway_bits = u32::from(gateway);
    if gateway_bits & mask_for(prefix) != u32::from(network) {
        return Err(format!("gateway {gateway_ip} is outside subnet {subnet_cidr}"));
    }
    let mut blocks = Vec::new();
    cidr_excluding(u32::from(network), prefix, gateway_bits, &mut blocks);
    Ok(blocks)
}

/// Build the Incus ACL name for a bridge subnet. Incus names allow only
/// ASCII letters, digits and dashes, and must not start with a digit or dash.
pub(crate) fn acl_name(subnet_cidr: &str) -> Result<String, String> {
    let suffix: String = subnet_cidr
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let name = format!("{ISOLATION_ACL_PREFIX}{suffix}");
    if name.len() > 63 || name.ends_with('-') {
        return Err(format!("isolation ACL name '{name}' is invalid"));
    }
    Ok(name)
}

/// One ordered ACL rule: the direction (`ingress`/`egress`) and the
/// `key=value` properties handed to `incus network acl rule add`.
pub(crate) struct AclRule {
    pub(crate) direction: &'static str,
    pub(crate) properties: String,
}

/// Build the ACL rules. Drops cover every tenant address but the gateway;
/// explicit allows keep the gateway and off-bridge internet reachable.
/// Unmatched traffic (notably IPv6) falls through to Incus' default reject.
pub(crate) fn isolation_rules(gateway_ip: &str, subnet_cidr: &str) -> Result<Vec<AclRule>, String> {
    let blocks = tenant_blocks(subnet_cidr, gateway_ip)?;
    let mut rules = Vec::with_capacity(blocks.len() * 2 + 2);
    for block in &blocks {
        rules.push(AclRule {
            direction: "ingress",
            properties: format!("action=drop,source={block}"),
        });
        rules.push(AclRule {
            direction: "egress",
            properties: format!("action=drop,destination={block}"),
        });
    }
    rules.push(AclRule {
        direction: "ingress",
        properties: "action=allow,source=0.0.0.0/0".to_string(),
    });
    rules.push(AclRule {
        direction: "egress",
        properties: "action=allow,destination=0.0.0.0/0".to_string(),
    });
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_subnet_derives_gateway_and_subnet() {
        let (gateway, subnet) = bridge_subnet("10.123.45.1/24").expect("valid bridge CIDR");
        assert_eq!(gateway, "10.123.45.1");
        assert_eq!(subnet, "10.123.45.0/24");
    }

    #[test]
    fn bridge_subnet_rejects_malformed_cidr() {
        assert!(bridge_subnet("10.123.45.1").is_err());
        assert!(bridge_subnet("not-an-ip/24").is_err());
        assert!(bridge_subnet("10.0.0.1/33").is_err());
    }

    #[test]
    fn cross_tenant_destination_is_dropped() {
        let decision = destination_allowed("10.123.45.9", "10.123.45.1", "10.123.45.0/24")
            .expect("valid destination");
        assert_eq!(decision, IsolationDecision::Drop);
    }

    #[test]
    fn gateway_and_internet_are_allowed() {
        let gateway = destination_allowed("10.123.45.1", "10.123.45.1", "10.123.45.0/24")
            .expect("valid gateway");
        assert_eq!(gateway, IsolationDecision::Allow);
        let internet = destination_allowed("8.8.8.8", "10.123.45.1", "10.123.45.0/24")
            .expect("valid internet destination");
        assert_eq!(internet, IsolationDecision::Allow);
    }

    #[test]
    fn probe_ip_is_a_non_gateway_tenant_address() {
        let probe = bridge_probe_ip("10.123.45.0/24", "10.123.45.1").expect("valid subnet");
        assert_eq!(probe, "10.123.45.2");
        assert_eq!(
            destination_allowed(&probe, "10.123.45.1", "10.123.45.0/24").expect("valid probe"),
            IsolationDecision::Drop
        );
        assert!(bridge_probe_ip("10.0.0.0/31", "10.0.0.1").is_err());
    }

    #[test]
    fn tenant_blocks_cover_subnet_except_gateway() {
        let blocks = tenant_blocks("10.123.45.0/24", "10.123.45.1").expect("valid subnet");
        assert_eq!(blocks.len(), 8);
        assert!(blocks.contains(&"10.123.45.0/32".to_string()));
        assert!(blocks.contains(&"10.123.45.2/31".to_string()));
        assert!(blocks.contains(&"10.123.45.128/25".to_string()));
        for block in &blocks {
            assert_ne!(block, "10.123.45.1/32");
        }
        assert!(tenant_blocks("10.0.0.0/24", "10.1.0.1").is_err());
    }

    #[test]
    fn acl_name_is_incus_compatible_and_subnet_scoped() {
        let name = acl_name("10.123.45.0/24").expect("valid subnet");
        assert_eq!(name, "vibe-iso-10-123-45-0-24");
        assert!(!name.starts_with('-') && !name.ends_with('-'));
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn acl_rules_drop_tenants_and_allow_gateway_and_internet() {
        let rules = isolation_rules("10.123.45.1", "10.123.45.0/24").expect("valid subnet");
        assert_eq!(rules.len(), 18);
        assert_eq!(rules[0].direction, "ingress");
        assert_eq!(rules[0].properties, "action=drop,source=10.123.45.128/25");
        assert_eq!(rules[16].properties, "action=allow,source=0.0.0.0/0");
        assert_eq!(rules[17].properties, "action=allow,destination=0.0.0.0/0");
    }
}
