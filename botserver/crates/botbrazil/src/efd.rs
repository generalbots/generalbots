//! EFD (Escrituração Fiscal Digital) helpers. SPED-related but EFD-specific
//! (EFD ICMS IPI, EFD Contribuições).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EfdKind {
    EfdIcmsIpi,
    EfdContribuicoes,
    EfdReinf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfdLine {
    pub register: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfdFile {
    pub kind: EfdKind,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub cnpj: String,
    pub lines: Vec<EfdLine>,
}

impl EfdFile {
    pub fn new(
        kind: EfdKind,
        period_start: NaiveDate,
        period_end: NaiveDate,
        cnpj: String,
    ) -> Self {
        Self {
            kind,
            period_start,
            period_end,
            cnpj,
            lines: Vec::new(),
        }
    }

    pub fn push(&mut self, line: EfdLine) {
        self.lines.push(line);
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for line in &self.lines {
            out.push('|');
            out.push_str(&line.register);
            for field in &line.fields {
                out.push('|');
                out.push_str(field);
            }
            out.push_str("|\n");
        }
        out
    }
}

pub fn efd_open(kind: EfdKind, period_start: NaiveDate, period_end: NaiveDate, cnpj: String) -> EfdFile {
    EfdFile::new(kind, period_start, period_end, cnpj)
}

pub fn efd_register_0(period_start: NaiveDate, period_end: NaiveDate, cnpj: &str) -> EfdLine {
    EfdLine {
        register: "0000".into(),
        fields: vec![
            period_start.format("%d%m%Y").to_string(),
            period_end.format("%d%m%Y").to_string(),
            cnpj.into(),
        ],
    }
}

pub fn efd_register_9001(indicator: &str) -> EfdLine {
    EfdLine {
        register: "9001".into(),
        fields: vec![indicator.into()],
    }
}

pub fn efd_register_9999(line_count: u32) -> EfdLine {
    EfdLine {
        register: "9999".into(),
        fields: vec![line_count.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn render_efd_basic() {
        let mut file = efd_open(
            EfdKind::EfdIcmsIpi,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            "11222333000181".into(),
        );
        file.push(efd_register_0(
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            "11222333000181",
        ));
        file.push(efd_register_9001("0"));
        file.push(efd_register_9999(3));
        let out = file.render();
        assert!(out.contains("|0000|"));
        assert!(out.contains("|9001|0|"));
        assert!(out.contains("|9999|3|"));
    }
}
