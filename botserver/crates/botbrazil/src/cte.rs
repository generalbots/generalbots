//! CT-e (Conhecimento de Transporte eletrônico) helpers.

use crate::models::{DocumentStatus, FiscalDocument, Party};
use crate::validators::TaxError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CteModal {
    Road,
    Air,
    Water,
    Rail,
    Pipeline,
}

#[derive(Debug, Clone)]
pub struct CteDraft {
    pub document: FiscalDocument,
    pub modal: CteModal,
    pub carrier: Party,
    pub origin: String,
    pub destination: String,
}

pub fn build_cte(
    mut document: FiscalDocument,
    modal: CteModal,
    carrier: Party,
    origin: String,
    destination: String,
) -> Result<CteDraft, TaxError> {
    if document.status != DocumentStatus::Draft {
        return Err(TaxError::InvalidState(format!("{:?}", document.status)));
    }
    if origin.is_empty() || destination.is_empty() {
        return Err(TaxError::MissingField("origin/destination"));
    }
    if document.items.is_empty() {
        return Err(TaxError::MissingField("items"));
    }
    document.recalculate_totals();
    Ok(CteDraft {
        document,
        modal,
        carrier,
        origin,
        destination,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Address, Party};

    fn party(tax_id: &str) -> Party {
        Party {
            tax_id: tax_id.into(),
            name: "Carrier".into(),
            legal_name: None,
            email: None,
            phone: None,
            address: Some(Address {
                street: "Rua B".into(),
                number: "2".into(),
                complement: None,
                district: "Centro".into(),
                city_code_ibge: "3550308".into(),
                city: "Sao Paulo".into(),
                state: "SP".into(),
                zip_code: "01000-000".into(),
                country_code: "1058".into(),
                country: "Brasil".into(),
            }),
            is_ie: true,
            is_icms_taxpayer: true,
        }
    }

    fn draft() -> FiscalDocument {
        let mut doc = FiscalDocument::new(party("11222333000181"), party(""), crate::models::DocumentKind::CTe);
        doc.add_item(crate::models::InvoiceItem {
            sku: "C1".into(),
            description: "Carga".into(),
            ncm: None,
            cfop: Some("6352".into()),
            unit: "KG".into(),
            quantity: 100.0,
            unit_price: 5.0,
            total_price: 500.0,
            taxes: vec![],
            origin: 0,
        });
        doc
    }

    #[test]
    fn build_cte_ok() {
        let draft = build_cte(
            draft(),
            CteModal::Road,
            party("11222333000181"),
            "3550308".into(),
            "3304557".into(),
        )
        .expect("ok");
        assert_eq!(draft.modal, CteModal::Road);
    }

    #[test]
    fn build_cte_requires_route() {
        let result = build_cte(draft(), CteModal::Road, party(""), String::new(), "".into());
        assert!(matches!(result, Err(TaxError::MissingField("origin/destination"))));
    }
}
