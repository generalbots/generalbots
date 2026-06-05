//! NFe (Nota Fiscal eletrônica) building block. Validates the document body
//! before sending it to the SEFAZ web service. No real network call is
//! performed — that is delegated to the integration layer that owns the
//! digital certificate.

use crate::models::{DocumentStatus, FiscalDocument};
use crate::validators::{is_valid_cnpj, is_valid_nfe_access_key, TaxError};

pub struct NFeDraft {
    pub document: FiscalDocument,
    pub items_total: f64,
    pub tax_total: f64,
}

pub fn build_nfe(mut document: FiscalDocument) -> Result<NFeDraft, TaxError> {
    if !is_valid_cnpj(&document.issuer.tax_id) {
        return Err(TaxError::InvalidCnpj(document.issuer.tax_id.clone()));
    }
    if !document.recipient.tax_id.is_empty() && !is_valid_cnpj(&document.recipient.tax_id) {
        return Err(TaxError::InvalidCnpj(document.recipient.tax_id.clone()));
    }
    if document.document_number.is_empty() {
        return Err(TaxError::MissingField("document_number"));
    }
    if document.series.is_empty() {
        return Err(TaxError::MissingField("series"));
    }
    if document.items.is_empty() {
        return Err(TaxError::MissingField("items"));
    }
    if document.status != DocumentStatus::Draft {
        return Err(TaxError::InvalidState(format!("{:?}", document.status)));
    }
    document.recalculate_totals();
    Ok(NFeDraft {
        items_total: document.total_products,
        tax_total: document.total_taxes,
        document,
    })
}

pub fn build_access_key(
    state: &str,
    issue_year: u16,
    issue_month: u8,
    cnpj: &str,
    model: &str,
    series: &str,
    number: u32,
    emission_kind: &str,
    code: u32,
) -> Result<String, TaxError> {
    let cnpj_digits = cnpj.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
    if cnpj_digits.len() != 14 {
        return Err(TaxError::InvalidCnpj(cnpj.to_string()));
    }
    let key = format!(
        "{}{:02}{}{}{}{:09}{:03}{}",
        state,
        issue_year % 100,
        issue_month,
        &cnpj_digits,
        model,
        emission_kind,
        series
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0),
        number,
        code,
    );
    let mut with_dv = key.clone();
    let bytes: Vec<u8> = key.bytes().map(|b| b - b'0').collect();
    let weights: [u8; 43] = std::array::from_fn(|i| (43 - i) as u8);
    let sum: u32 = bytes
        .iter()
        .zip(weights.iter())
        .map(|(d, w)| (*d as u32) * (*w as u32))
        .sum();
    let remainder = sum % 11;
    let dv = if remainder < 2 { 0 } else { (11 - remainder) as u8 };
    with_dv.push((dv + b'0') as char);
    Ok(with_dv)
}

pub fn validate_nfe_key(key: &str) -> Result<(), TaxError> {
    if !is_valid_nfe_access_key(key) {
        return Err(TaxError::InvalidAccessKey(key.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Address, Party};

    fn party(tax_id: &str) -> Party {
        Party {
            tax_id: tax_id.into(),
            name: "Issuer".into(),
            legal_name: None,
            email: None,
            phone: None,
            address: Some(Address {
                street: "Av Paulista".into(),
                number: "1000".into(),
                complement: None,
                district: "Bela Vista".into(),
                city_code_ibge: "3550308".into(),
                city: "Sao Paulo".into(),
                state: "SP".into(),
                zip_code: "01310-100".into(),
                country_code: "1058".into(),
                country: "Brasil".into(),
            }),
            is_ie: true,
            is_icms_taxpayer: true,
        }
    }

    fn draft() -> FiscalDocument {
        let mut doc = FiscalDocument::new(party("11222333000181"), party(""), crate::models::DocumentKind::NFe);
        doc.document_number = "1".into();
        doc.series = "1".into();
        doc.add_item(crate::models::InvoiceItem {
            sku: "SKU1".into(),
            description: "Item 1".into(),
            ncm: Some("84715010".into()),
            cfop: Some("6102".into()),
            unit: "UN".into(),
            quantity: 1.0,
            unit_price: 100.0,
            total_price: 100.0,
            taxes: vec![],
            origin: 0,
        });
        doc
    }

    #[test]
    fn build_nfe_ok() {
        let draft = build_nfe(draft()).expect("valid NFe draft");
        assert_eq!(draft.items_total, 100.0);
    }

    #[test]
    fn build_nfe_rejects_invalid_cnpj() {
        let mut d = draft();
        d.issuer.tax_id = "123".into();
        let result = build_nfe(d);
        assert!(matches!(result, Err(TaxError::InvalidCnpj(_))));
    }

    #[test]
    fn build_nfe_rejects_missing_number() {
        let mut d = draft();
        d.document_number = String::new();
        let result = build_nfe(d);
        assert!(matches!(result, Err(TaxError::MissingField("document_number"))));
    }

    #[test]
    fn access_key_is_44_digits() {
        let key = build_access_key("35", 2026, 6, "11222333000181", "55", "1", 1, "1", 0).expect("key");
        assert_eq!(key.len(), 44);
        assert!(is_valid_nfe_access_key(&key));
    }
}
