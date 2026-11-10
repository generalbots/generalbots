use super::*;
use x509_parser::prelude::FromDer;

/// Extracts the certificate serial number as an uppercase hex string.
///
/// Revocations are stored by serial (`serial|reason|date`), so verification
/// has to compare the same representation instead of hashing the PEM (which
/// nothing ever wrote to the revocation file).
fn certificate_serial(cert_pem: &str) -> Option<String> {
    let der = pem_to_der(cert_pem).ok()?;
    let (_, cert) = x509_parser::certificate::X509Certificate::from_der(&der).ok()?;

    Some(cert.raw_serial_as_string().to_uppercase())
}

fn pem_to_der(cert_pem: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let start = cert_pem
        .find("-----BEGIN CERTIFICATE-----")
        .ok_or_else(|| anyhow::anyhow!("missing BEGIN CERTIFICATE marker"))?;
    let end = cert_pem
        .find("-----END CERTIFICATE-----")
        .ok_or_else(|| anyhow::anyhow!("missing END CERTIFICATE marker"))?;

    let body: String = cert_pem[start + "-----BEGIN CERTIFICATE-----".len()..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    STANDARD
        .decode(body)
        .map_err(|e| anyhow::anyhow!("invalid base64 certificate: {e}"))
}

impl CaManager {
    pub fn verify_certificate(&self, cert_pem: &str) -> Result<bool> {
        if !self.config.ca_cert_path.exists() {
            debug!("CA certificate not found");
            return Ok(false);
        }

        if cert_pem.is_empty() || !cert_pem.contains("BEGIN CERTIFICATE") {
            debug!("Invalid certificate PEM format");
            return Ok(false);
        }

        let revoked_path = self.config.ca_cert_path.with_extension("revoked");
        if revoked_path.exists() {
            let revoked_content = fs::read_to_string(&revoked_path)?;

            if let Some(serial) = certificate_serial(cert_pem) {
                let is_revoked = revoked_content.lines().any(|line| {
                    line.split('|')
                        .next()
                        .map(|listed| listed.trim().eq_ignore_ascii_case(&serial))
                        .unwrap_or(false)
                });

                if is_revoked {
                    debug!("Certificate serial {} is revoked", serial);
                    return Ok(false);
                }
            }
        }

        info!("Certificate verified successfully");
        Ok(true)
    }

    pub fn revoke_certificate(&self, serial_number: &str, reason: &str) -> Result<()> {
        let revoked_path = self.config.ca_cert_path.with_extension("revoked");

        let entry = format!(
            "{}|{}|{}\n",
            serial_number,
            reason,
            OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?
        );

        let mut content = if revoked_path.exists() {
            fs::read_to_string(&revoked_path)?
        } else {
            String::new()
        };

        content.push_str(&entry);
        fs::write(&revoked_path, content)?;

        info!("Certificate {} revoked. Reason: {}", serial_number, reason);

        self.generate_crl()?;

        Ok(())
    }

    pub fn generate_crl(&self) -> Result<()> {
        let revoked_path = self.config.ca_cert_path.with_extension("revoked");
        let crl_path = self.config.ca_cert_path.with_extension("crl");

        let mut crl_content = String::from("-----BEGIN X509 CRL-----\n");
        let _ = writeln!(
            crl_content,
            "# CRL Generated: {}",
            OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?
        );
        let _ = writeln!(crl_content, "# Issuer: {}", self.config.organization);

        if revoked_path.exists() {
            let revoked = fs::read_to_string(&revoked_path)?;
            for line in revoked.lines() {
                if !line.is_empty() {
                    let _ = writeln!(crl_content, "# Revoked: {}", line);
                }
            }
        }

        crl_content.push_str("-----END X509 CRL-----\n");
        fs::write(&crl_path, crl_content)?;

        info!("CRL generated at {:?}", crl_path);

        Ok(())
    }

    pub async fn sync_with_external_ca(&self) -> Result<()> {
        if !self.config.external_ca_enabled {
            return Ok(());
        }

        let (Some(url), Some(api_key)) = (
            &self.config.external_ca_url,
            &self.config.external_ca_api_key,
        ) else {
            return Ok(());
        };

        info!("Syncing with external CA at {}", url);

        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/status", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            info!("External CA sync successful");
        } else {
            warn!("External CA returned status: {}", response.status());
        }

        Ok(())
    }
}
