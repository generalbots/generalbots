//! The [`Registrar`] implementation for Namecheap.
//!
//! Split from the adapter definition to keep both files inside the project size
//! limit. The hazard this file exists to contain is documented on
//! [`super::Namecheap`].

use async_trait::async_trait;

use super::{Namecheap, SERVICE, nameserver_param, split_domain};
use crate::types::{
    Availability, Credential, DnsRecord, DomainSpec, RegistrarError, RegistrarInfo, Registration,
    TldPrice,
};
use crate::{Registrar, xml};

#[async_trait]
impl Registrar for Namecheap {
    fn name(&self) -> &str {
        "namecheap"
    }

    fn info(&self) -> RegistrarInfo {
        RegistrarInfo {
            name: "namecheap".into(),
            display_name: "Namecheap".into(),
            supported_tlds: vec![
                "com".into(), "net".into(), "org".into(), "io".into(), "ai".into(), "co".into(),
                "dev".into(), "app".into(), "xyz".into(),
            ],
            manages_dns: true,
            // The property that shapes this whole adapter.
            replaces_whole_zone: true,
            manages_dnssec: false,
        }
    }

    async fn check(&self, domain: &str, credential: &Credential) -> Result<Availability, RegistrarError> {
        let (_, tld) = split_domain(domain)?;
        let parsed = self
            .call(
                "namecheap.domains.check",
                credential,
                &[("domainList", tld.to_string()), ("checkType", "REGISTER".to_string())],
            )
            .await?;

        let flags = xml::availability_flags(&parsed);
        Ok(match flags.first().map(|f| f.to_lowercase()) {
            Some(flag) if flag == "true" => Availability::Available { premium: false, price_cents: None },
            Some(flag) if flag == "false" => Availability::Taken,
            // A response with no availability flag is not a "yes": treating the
            // absence as available would let the platform attempt a purchase it
            // knows nothing about.
            _ => Availability::Reserved,
        })
    }

    async fn register(&self, spec: &DomainSpec, credential: &Credential) -> Result<Registration, RegistrarError> {
        let (label, tld) = split_domain(&spec.domain)?;
        let domain = format!("{label}.{tld}");
        let mut extra: Vec<(&str, String)> = vec![("domain", domain.clone())];
        if spec.whois_privacy {
            extra.push(("addFreeWhoisguard", "yes".to_string()));
        }
        self.call("namecheap.domains.create", credential, &extra).await?;

        if !spec.nameservers.is_empty() {
            self.set_nameservers(&domain, &spec.nameservers, credential).await?;
        }
        Ok(Registration {
            domain,
            registrar_id: None,
            expires_at: None,
            auto_renew: spec.auto_renew,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn renew(&self, domain: &str, _years: u32, credential: &Credential) -> Result<Registration, RegistrarError> {
        let domain = domain.trim().to_lowercase();
        self.call("namecheap.domains.renew", credential, &[("domain", domain.clone())]).await?;
        Ok(Registration {
            domain,
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn transfer(&self, domain: &str, credential: &Credential) -> Result<Registration, RegistrarError> {
        let domain = domain.trim().to_lowercase();
        self.call(
            "namecheap.domains.transfer.initiate",
            credential,
            &[("domain", domain.clone())],
        )
        .await?;
        Ok(Registration {
            domain,
            registrar_id: None,
            expires_at: None,
            auto_renew: true,
            charged: None,
            currency: Some("USD".into()),
        })
    }

    async fn set_nameservers(&self, domain: &str, nameservers: &[String], credential: &Credential) -> Result<(), RegistrarError> {
        let mut extra: Vec<(&str, String)> = vec![("domain", domain.trim().to_lowercase())];
        for (index, ns) in nameservers.iter().take(4).enumerate() {
            extra.push((nameserver_param(index), ns.clone()));
        }
        self.call("namecheap.domains.dns.setCustom", credential, &extra).await?;
        Ok(())
    }

    async fn get_dns_record(&self, domain: &str, host: &str, credential: &Credential) -> Result<Option<DnsRecord>, RegistrarError> {
        let wanted = host.trim().to_lowercase();
        Ok(self
            .zone(domain, credential)
            .await?
            .into_iter()
            .find(|record| record.host.trim().to_lowercase() == wanted))
    }

    async fn set_dns_record(&self, domain: &str, record: &DnsRecord, credential: &Credential) -> Result<DnsRecord, RegistrarError> {
        let mut desired = self.zone(domain, credential).await?;
        // Drop any existing record with the same (host, type) so the diff reads as
        // a change rather than as round-robin growth.
        desired.retain(|existing| {
            existing.host.trim().to_lowercase() != record.host.trim().to_lowercase()
                || existing.record_type != record.record_type
        });
        let stored = DnsRecord { record_id: None, ..record.clone() };
        desired.push(stored.clone());
        self.replace_zone(domain, &desired, credential).await?;
        Ok(stored)
    }

    async fn delete_dns_record(&self, _domain: &str, _record_id: &str, _credential: &Credential) -> Result<(), RegistrarError> {
        // There is no per-record delete. "Delete by id" is simply not expressible
        // against an API whose only write is destructive, and pretending
        // otherwise would risk the customer's whole zone.
        Err(RegistrarError::Unsupported(
            SERVICE.into(),
            "this API has no per-record delete; set_dns_record replaces the zone instead".into(),
        ))
    }

    async fn tld_list(&self, _credential: &Credential) -> Result<Vec<TldPrice>, RegistrarError> {
        Ok(self
            .info()
            .supported_tlds
            .iter()
            .map(|tld| TldPrice {
                tld: tld.clone(),
                registration_cents: 1_098,
                // The steepest renewal markup in the adapter set, and one of the
                // reasons this is not the default registrar.
                renewal_cents: 1_848,
                premium: false,
            })
            .collect())
    }
}
