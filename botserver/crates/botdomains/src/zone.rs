//! Zone arithmetic for registrars whose DNS write replaces the entire zone.
//!
//! Namecheap's only DNS write endpoint is `domains.dns.setHosts`, which
//! documents itself as replacing the full record set. There is no per-record
//! create or delete. That makes every single-record change a read-all,
//! modify-in-memory, write-all cycle — and any record the client forgets to
//! re-send is silently deleted from the customer's zone.
//!
//! This module makes that failure impossible to reach by accident: a change is
//! computed as an explicit [`ZoneDiff`] and the caller must acknowledge the
//! records that would be destroyed before the write is allowed.

use crate::types::{DnsRecord, RecordType};

/// Identity of a record inside a zone: the (host, type) pair, which is what DNS
/// treats as unique. Two `A` records for the same host are a round-robin set, not
/// a conflict, so the diff groups on this key rather than the value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordKey {
    pub host: String,
    pub record_type: RecordType,
}

impl RecordKey {
    #[must_use]
    pub fn new(host: &str, record_type: RecordType) -> Self {
        Self { host: host.trim().to_lowercase(), record_type }
    }

    #[must_use]
    pub fn of(record: &DnsRecord) -> Self {
        Self::new(&record.host, record.record_type)
    }
}

/// What a proposed write would do to a zone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ZoneDiff {
    /// Records absent from the current zone that the write would add.
    pub added: Vec<DnsRecord>,
    /// Record groups whose value or TTL would change: `(before, after)`.
    pub changed: Vec<(Vec<DnsRecord>, Vec<DnsRecord>)>,
    /// Records present in the zone that the write would drop.
    pub removed: Vec<DnsRecord>,
}

impl ZoneDiff {
    /// True when the write leaves the zone exactly as it is.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.changed.is_empty() && self.removed.is_empty()
    }

    /// Records a full-zone write would destroy.
    #[must_use]
    pub fn removals(&self) -> &[DnsRecord] {
        &self.removed
    }

    /// Refuses a write that would silently destroy records.
    ///
    /// This is the guard the Namecheap adapter applies. The alternative — a
    /// customer losing their MX records because the platform forgot to re-send
    /// them — is not recoverable from inside the adapter.
    pub fn ensure_no_silent_deletions(&self, service: &str) -> Result<(), String> {
        if self.removed.is_empty() {
            return Ok(());
        }
        let names: Vec<String> = self
            .removed
            .iter()
            .map(|record| {
                let host = if record.host.is_empty() { "@" } else { &record.host };
                format!("{host} {}", record.record_type.as_str())
            })
            .collect();
        Err(format!(
            "{service}: refusing to replace the zone — it would delete {} record(s): {}",
            names.len(),
            names.join(", ")
        ))
    }
}

/// Computes what turning `current` into `desired` would do.
///
/// Round-robins are compared as sets: a record is "changed" only when the same
/// (host, type) key ends up with a different set of values, so reordering a
/// round-robin is not reported as a change and is not rewritten.
#[must_use]
pub fn diff(current: &[DnsRecord], desired: &[DnsRecord]) -> ZoneDiff {
    let mut diff = ZoneDiff::default();

    let mut current_by_key: std::collections::BTreeMap<RecordKey, Vec<DnsRecord>> =
        std::collections::BTreeMap::new();
    for record in current {
        current_by_key.entry(RecordKey::of(record)).or_default().push(record.clone());
    }

    let mut desired_by_key: std::collections::BTreeMap<RecordKey, Vec<DnsRecord>> =
        std::collections::BTreeMap::new();
    for record in desired {
        desired_by_key.entry(RecordKey::of(record)).or_default().push(record.clone());
    }

    for (key, wanted) in &desired_by_key {
        match current_by_key.get(key) {
            None => diff.added.extend(wanted.iter().cloned()),
            Some(present) if !same_records(present, wanted) => {
                diff.changed.push((present.clone(), wanted.clone()));
            }
            Some(_) => {}
        }
    }

    for (key, present) in &current_by_key {
        if !desired_by_key.contains_key(key) {
            diff.removed.extend(present.iter().cloned());
        }
    }

    diff
}

/// Compares two record groups on the fields DNS cares about, order-independently.
fn same_records(left: &[DnsRecord], right: &[DnsRecord]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let key_of = |records: &[DnsRecord]| -> std::collections::BTreeSet<(String, u32)> {
        records
            .iter()
            .map(|record| (record.value.clone(), record.ttl))
            .collect()
    };
    key_of(left) == key_of(right)
}

/// The records a platform-owned domain needs at registration.
///
/// Beyond the routing A record, WhatsApp and email onboarding both fail without
/// their verification records, so they are written as part of the provision rather
/// than left as a follow-up ticket.
#[must_use]
pub fn onboarding_records(
    app_ip: &str,
    app_host: &str,
    dmarc_record: &str,
) -> Vec<DnsRecord> {
    vec![
        DnsRecord::new("", RecordType::A, app_ip),
        DnsRecord::new("www", RecordType::Cname, app_host),
        // CAA pins which CAs may issue for this zone. Only Let's Encrypt, which
        // is what the platform's ACME client uses.
        DnsRecord::new("", RecordType::Caa, "0 issue \"letsencrypt.org\""),
        DnsRecord::new("", RecordType::Caa, "0 iodef \"mailto:abuse@generalbots.org\""),
        // SPF authorises the platform's mail relay and nothing else.
        DnsRecord::new("", RecordType::Txt, "v=spf1 include:generalbots.org ~all"),
        DnsRecord::new("_dmarc", RecordType::Txt, dmarc_record),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(host: &str, value: &str) -> DnsRecord {
        DnsRecord::new(host, RecordType::A, value)
    }

    fn txt(host: &str, value: &str) -> DnsRecord {
        DnsRecord::new(host, RecordType::Txt, value)
    }

    #[test]
    fn an_unchanged_zone_produces_an_empty_diff() {
        let zone = vec![a("", "203.0.113.7"), txt("_dmarc", "v=DMARC1; p=none")];
        assert!(diff(&zone, &zone).is_empty());
    }

    #[test]
    fn a_new_record_is_an_addition() {
        let diff = diff(&[a("", "203.0.113.7")], &[a("", "203.0.113.7"), txt("v", "spf1")]);
        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
        assert!(diff.changed.is_empty());
    }

    #[test]
    fn a_dropped_record_is_a_removal_and_is_refused() {
        let before = vec![a("", "203.0.113.7"), txt("@mx", "mail.example.net")];
        let after = vec![a("", "203.0.113.7")];
        let diff = diff(&before, &after);
        assert_eq!(diff.removed.len(), 1);
        let err = diff.ensure_no_silent_deletions("Namecheap").unwrap_err();
        // The message names the record so an operator can find it in their zone.
        assert!(err.contains("@mx TXT"), "{err}");
        assert!(err.contains("Namecheap"), "{err}");
    }

    #[test]
    fn a_removal_guard_passes_a_pure_addition() {
        let diff = diff(&[], &[a("", "203.0.113.7")]);
        assert!(diff.ensure_no_silent_deletions("Namecheap").is_ok());
    }

    #[test]
    fn changing_a_value_is_reported_as_a_change_not_a_delete_plus_add() {
        let diff = diff(&[a("", "203.0.113.7")], &[a("", "203.0.113.9")]);
        assert_eq!(diff.changed.len(), 1);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty(), "an in-place change must not read as a deletion");
    }

    #[test]
    fn a_round_robin_reordering_is_not_a_change() {
        let before = vec![a("", "203.0.113.7"), a("", "203.0.113.8")];
        let after = vec![a("", "203.0.113.8"), a("", "203.0.113.7")];
        assert!(diff(&before, &after).is_empty());
    }

    #[test]
    fn a_grown_round_robin_is_a_change() {
        let before = vec![a("", "203.0.113.7")];
        let after = vec![a("", "203.0.113.7"), a("", "203.0.113.8")];
        let diff = diff(&before, &after);
        assert_eq!(diff.changed.len(), 1);
    }

    #[test]
    fn the_same_host_can_carry_different_types_independently() {
        let before = vec![a("", "203.0.113.7")];
        let after = vec![a("", "203.0.113.7"), txt("", "v=spf1 -all")];
        let diff = diff(&before, &after);
        assert_eq!(diff.added.len(), 1, "the TXT must not collide with the A record");
    }

    #[test]
    fn host_labels_compare_case_insensitively() {
        let before = vec![DnsRecord::new("WWW", RecordType::Cname, "target.example.net")];
        let after = vec![DnsRecord::new("www", RecordType::Cname, "target.example.net")];
        assert!(diff(&before, &after).is_empty());
    }

    #[test]
    fn onboarding_covers_routing_caa_spf_and_dmarc() {
        let records = onboarding_records("203.0.113.7", "app.example.net", "v=DMARC1; p=reject");
        let types: Vec<RecordType> = records.iter().map(|r| r.record_type).collect();
        assert!(types.contains(&RecordType::A));
        assert!(types.contains(&RecordType::Cname));
        assert_eq!(types.iter().filter(|t| **t == RecordType::Caa).count(), 2);
        assert!(types.contains(&RecordType::Txt));
        assert!(records.iter().any(|r| r.host == "_dmarc"));
    }

    #[test]
    fn the_apex_a_record_has_an_empty_host() {
        let records = onboarding_records("203.0.113.7", "app.example.net", "v=DMARC1; p=none");
        let apex = records.iter().find(|r| r.record_type == RecordType::A).expect("apex A");
        assert_eq!(apex.host, "", "the apex is an empty label, not \"@\"");
    }
}
