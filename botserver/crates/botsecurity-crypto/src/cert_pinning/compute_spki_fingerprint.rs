use super::*;

pub fn compute_spki_fingerprint(cert_der: &[u8]) -> Result<Vec<u8>> {
    let (_, cert) = X509Certificate::from_der(cert_der)
        .map_err(|e| anyhow!("Failed to parse X.509 certificate: {}", e))?;

    let spki = cert.public_key().raw;

    let hash = digest(&SHA256, spki);

    Ok(hash.as_ref().to_vec())
}
pub fn compute_cert_fingerprint(cert_der: &[u8]) -> Vec<u8> {
    let hash = digest(&SHA256, cert_der);
    hash.as_ref().to_vec()
}
pub fn pem_to_der(pem_data: &[u8]) -> Result<Vec<u8>> {
    let pem_str = std::str::from_utf8(pem_data).context("Invalid UTF-8 in PEM data")?;

    let start_marker = "-----BEGIN CERTIFICATE-----";
    let end_marker = "-----END CERTIFICATE-----";

    let start = pem_str
        .find(start_marker)
        .ok_or_else(|| anyhow!("No certificate found in PEM data"))?;

    let end = pem_str
        .find(end_marker)
        .ok_or_else(|| anyhow!("Invalid PEM: missing end marker"))?;

    let base64_data = &pem_str[start + start_marker.len()..end];
    let cleaned: String = base64_data.chars().filter(|c| !c.is_whitespace()).collect();

    BASE64
        .decode(&cleaned)
        .context("Failed to decode base64 certificate data")
}
pub fn format_fingerprint(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}
pub fn parse_fingerprint(formatted: &str) -> Result<Vec<u8>> {
    let trimmed = formatted.trim();

    if let Some(base64_part) = trimmed.strip_prefix("sha256//") {
        return BASE64
            .decode(base64_part.trim())
            .context("Failed to decode base64 fingerprint");
    }

    if trimmed.contains(':') {
        let bytes: Result<Vec<u8>, _> = trimmed
            .split(':')
            .map(|hex| u8::from_str_radix(hex.trim(), 16))
            .collect();

        return bytes.context("Failed to parse hex fingerprint");
    }

    let compact: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();

    if compact.is_empty() {
        return Err(anyhow!("Empty fingerprint"));
    }

    if compact.len() % 2 != 0 {
        return Err(anyhow!("Fingerprint has an odd number of hex digits"));
    }

    // `as_bytes().chunks(2)` keeps the parser from panicking on non-ASCII
    // input (a previous byte-slice version could split a multi-byte character
    // and abort the process).
    let bytes: Result<Vec<u8>, _> = compact
        .as_bytes()
        .chunks(2)
        .map(|pair| {
            let hex = std::str::from_utf8(pair).context("Fingerprint is not ASCII hex")?;
            u8::from_str_radix(hex, 16).context("Failed to parse fingerprint")
        })
        .collect();

    bytes.context("Failed to parse fingerprint")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_der() -> Vec<u8> {
        let cert = rcgen::generate_simple_self_signed(vec!["example.com".to_string()])
            .expect("self-signed certificate");
        cert.cert.der().to_vec()
    }

    #[test]
    fn format_fingerprint_uses_uppercase_colons() {
        assert_eq!(format_fingerprint(&[0x00, 0x0a, 0xff]), "00:0A:FF");
    }

    #[test]
    fn format_and_parse_round_trip() {
        let hash = [0xde, 0xad, 0xbe, 0xef];
        let formatted = format_fingerprint(&hash);
        assert_eq!(parse_fingerprint(&formatted).unwrap(), hash);
    }

    #[test]
    fn parse_accepts_compact_hex_and_whitespace() {
        assert_eq!(parse_fingerprint("DEADBEEF").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(
            parse_fingerprint(" de:ad:be:ef ").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
        assert_eq!(
            parse_fingerprint("de ad be ef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn parse_accepts_base64_sha256_prefix() {
        let raw = [1u8, 2, 3, 4];
        let formatted = format!("sha256//{}", BASE64.encode(raw));
        assert_eq!(parse_fingerprint(&formatted).unwrap(), raw);
    }

    #[test]
    fn parse_rejects_odd_length_without_panicking() {
        let err = parse_fingerprint("ABC").unwrap_err().to_string();
        assert!(err.contains("Fingerprint"), "unexpected error: {err}");
    }

    #[test]
    fn parse_rejects_empty_input() {
        assert!(parse_fingerprint("   ").is_err());
    }

    #[test]
    fn parse_rejects_non_ascii_without_panicking() {
        assert!(parse_fingerprint("éééé").is_err());
    }

    #[test]
    fn parse_rejects_invalid_hex_digits() {
        assert!(parse_fingerprint("zz:zz").is_err());
    }

    #[test]
    fn pem_to_der_decodes_a_real_certificate() {
        let cert = rcgen::generate_simple_self_signed(vec!["example.com".to_string()])
            .expect("self-signed certificate");
        let pem = cert.cert.pem();
        let der = pem_to_der(pem.as_bytes()).expect("PEM should decode");
        assert_eq!(der, cert.cert.der().to_vec());
    }

    #[test]
    fn pem_to_der_rejects_missing_markers() {
        assert!(pem_to_der(b"not a pem").is_err());
        assert!(pem_to_der(b"-----BEGIN CERTIFICATE-----\nAAAA").is_err());
    }

    #[test]
    fn spki_fingerprint_is_sha256_sized_and_stable() {
        let der = sample_der();
        let first = compute_spki_fingerprint(&der).expect("fingerprint");
        let second = compute_spki_fingerprint(&der).expect("fingerprint");
        assert_eq!(first.len(), 32);
        assert_eq!(first, second);
        assert_ne!(first, compute_cert_fingerprint(&der));
    }

    #[test]
    fn spki_fingerprint_rejects_garbage() {
        assert!(compute_spki_fingerprint(&[0x30, 0x00]).is_err());
    }
}
