//! MDF-e (Manifesto Eletrônico de Documentos Fiscais) helpers.

use crate::models::{DocumentStatus, FiscalDocument};
use crate::validators::TaxError;

#[derive(Debug, Clone)]
pub struct MdfeDraft {
    pub document: FiscalDocument,
    pub references: Vec<String>,
    pub vehicle_plate: String,
}

pub fn build_mdfe(
    document: FiscalDocument,
    references: Vec<String>,
    vehicle_plate: String,
) -> Result<MdfeDraft, TaxError> {
    if document.status != DocumentStatus::Draft {
        return Err(TaxError::InvalidState(format!("{:?}", document.status)));
    }
    if vehicle_plate.is_empty() {
        return Err(TaxError::MissingField("vehicle_plate"));
    }
    if references.is_empty() {
        return Err(TaxError::MissingField("references"));
    }
    Ok(MdfeDraft {
        document,
        references,
        vehicle_plate,
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
                street: "Rua".into(),
                number: "1".into(),
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
        FiscalDocument::new(party("11222333000181"), party(""), crate::models::DocumentKind::MDFe)
    }

    #[test]
    fn build_mdfe_ok() {
        let mdfe = build_mdfe(
            draft(),
            vec!["NFe-1".into()],
            "ABC1D23".into(),
        )
        .expect("ok");
        assert_eq!(mdfe.references.len(), 1);
    }

    #[test]
    fn build_mdfe_requires_plate() {
        let result = build_mdfe(draft(), vec!["NFe-1".into()], String::new());
        assert!(matches!(result, Err(TaxError::MissingField("vehicle_plate"))));
    }

    #[test]
    fn build_mdfe_requires_references() {
        let result = build_mdfe(draft(), vec![], "ABC1D23".into());
        assert!(matches!(result, Err(TaxError::MissingField("references"))));
    }
}
