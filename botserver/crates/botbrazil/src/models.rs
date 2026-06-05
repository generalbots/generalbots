//! Domain models for Brazilian electronic fiscal documents.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentKind {
    NFe,
    NFSe,
    CTe,
    MDFe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentStatus {
    Draft,
    Signed,
    Authorized,
    Rejected,
    Cancelled,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaxRegime {
    SimplesNacional,
    LucroPresumido,
    LucroReal,
    LucroArbitrado,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub number: String,
    pub complement: Option<String>,
    pub district: String,
    pub city_code_ibge: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country_code: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub tax_id: String,
    pub name: String,
    pub legal_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<Address>,
    pub is_ie: bool,
    pub is_icms_taxpayer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLine {
    pub tax_code: String,
    pub cst: String,
    pub base_amount: f64,
    pub tax_rate: f64,
    pub tax_amount: f64,
    pub base_reduction: Option<f64>,
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub sku: String,
    pub description: String,
    pub ncm: Option<String>,
    pub cfop: Option<String>,
    pub unit: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total_price: f64,
    pub taxes: Vec<TaxLine>,
    pub origin: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerm {
    pub method: String,
    pub amount: f64,
    pub due_date: Option<NaiveDate>,
    pub installment: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalDocument {
    pub id: Uuid,
    pub kind: DocumentKind,
    pub document_number: String,
    pub series: String,
    pub access_key: Option<String>,
    pub protocol: Option<String>,
    pub status: DocumentStatus,
    pub issuer: Party,
    pub recipient: Party,
    pub issue_date: DateTime<Utc>,
    pub entry_exit_date: Option<NaiveDate>,
    pub items: Vec<InvoiceItem>,
    pub payments: Vec<PaymentTerm>,
    pub total_products: f64,
    pub total_taxes: f64,
    pub total_invoice: f64,
    pub tax_regime: TaxRegime,
    pub extra_info: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FiscalDocument {
    pub fn new(issuer: Party, recipient: Party, kind: DocumentKind) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            kind,
            document_number: String::new(),
            series: "1".into(),
            access_key: None,
            protocol: None,
            status: DocumentStatus::Draft,
            issuer,
            recipient,
            issue_date: now,
            entry_exit_date: None,
            items: Vec::new(),
            payments: Vec::new(),
            total_products: 0.0,
            total_taxes: 0.0,
            total_invoice: 0.0,
            tax_regime: TaxRegime::SimplesNacional,
            extra_info: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_item(&mut self, item: InvoiceItem) {
        self.total_products += item.total_price;
        self.total_taxes += item.taxes.iter().map(|t| t.tax_amount).sum::<f64>();
        self.items.push(item);
    }

    pub fn recalculate_totals(&mut self) {
        self.total_products = self.items.iter().map(|i| i.total_price).sum();
        self.total_taxes = self
            .items
            .iter()
            .flat_map(|i| i.taxes.iter())
            .map(|t| t.tax_amount)
            .sum();
        self.total_invoice = self.total_products + self.total_taxes;
        self.updated_at = Utc::now();
    }
}
