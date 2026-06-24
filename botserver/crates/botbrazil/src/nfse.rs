//! NFSe (Nota Fiscal de Serviços eletrônica) helpers.
//!
//! NFSe is intrinsically municipal, so the helpers are intentionally minimal:
//! they validate the document, check the municipal code, and format the
//! RPS (Recibo Provisório de Serviços) number used by most providers.

use crate::models::{DocumentStatus, FiscalDocument};
use crate::validators::TaxError;

pub struct NFSeDraft {
    pub document: FiscalDocument,
    pub rps: Option<String>,
}

pub fn build_nfse(mut document: FiscalDocument, rps: Option<String>) -> Result<NFSeDraft, TaxError> {
    if document.status != DocumentStatus::Draft {
        return Err(TaxError::InvalidState(format!("{:?}", document.status)));
    }
    let issuer_address = document
        .issuer
        .address
        .as_ref()
        .ok_or(TaxError::MissingField("issuer.address"))?;
    if issuer_address.city_code_ibge.len() != 7 {
        return Err(TaxError::MissingField("issuer.address.city_code_ibge"));
    }
    if document.items.is_empty() {
        return Err(TaxError::MissingField("items"));
    }
    document.recalculate_totals();
    Ok(NFSeDraft { document, rps })
}

pub fn format_rps(number: u32, series: &str) -> String {
    format!("RPS-{}-{:09}", series, number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Address, Party};

    fn party(tax_id: &str, city_code: &str) -> Party {
        Party {
            tax_id: tax_id.into(),
            name: "Issuer".into(),
            legal_name: None,
            email: None,
            phone: None,
            address: Some(Address {
                street: "Rua A".into(),
                number: "1".into(),
                complement: None,
                district: "Centro".into(),
                city_code_ibge: city_code.into(),
                city: "Sao Paulo".into(),
                state: "SP".into(),
                zip_code: "01000-000".into(),
                country_code: "1058".into(),
                country: "Brasil".into(),
            }),
            is_ie: true,
            is_icms_taxpayer: false,
        }
    }

    fn draft() -> FiscalDocument {
        let mut doc = FiscalDocument::new(
            party("11222333000181", "3550308"),
            party("", ""),
            crate::models::DocumentKind::NFSe,
        );
        doc.add_item(crate::models::InvoiceItem {
            sku: "S1".into(),
            description: "Servico".into(),
            ncm: None,
            cfop: None,
            unit: "SV".into(),
            quantity: 1.0,
            unit_price: 200.0,
            total_price: 200.0,
            taxes: vec![],
            origin: 0,
        });
        doc
    }

    #[test]
    fn build_nfse_ok() {
        let draft = build_nfse(draft(), Some("RPS-1-000000001".into())).expect("ok");
        assert_eq!(draft.rps.as_deref(), Some("RPS-1-000000001"));
    }

    #[test]
    fn build_nfse_requires_municipality() {
        let mut d = draft();
        d.issuer.address = None;
        let result = build_nfse(d, None);
        assert!(matches!(result, Err(TaxError::MissingField("issuer.address"))));
    }

    #[test]
    fn rps_format() {
        assert_eq!(format_rps(1, "A"), "RPS-A-000000001");
    }
}
