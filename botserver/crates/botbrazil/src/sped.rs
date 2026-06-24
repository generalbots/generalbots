//! SPED (Sistema Público de Escrituração Digital) aggregator. Provides
//! file-level helpers to assemble and serialise a SPED contribution record.

use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpedProfile {
    Fiscal,
    Contribuicoes,
    Contabil,
    ECD,
    ECF,
}

#[derive(Debug, Clone)]
pub struct SpedFile {
    pub profile: SpedProfile,
    pub cnpj: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub blocks: Vec<SpedBlock>,
}

#[derive(Debug, Clone)]
pub struct SpedBlock {
    pub code: char,
    pub records: Vec<Vec<String>>,
}

impl SpedFile {
    pub fn new(
        profile: SpedProfile,
        cnpj: String,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Self {
        Self {
            profile,
            cnpj,
            period_start,
            period_end,
            blocks: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: SpedBlock) {
        if !self.blocks.iter().any(|b| b.code == block.code) {
            self.blocks.push(block);
        }
    }

    pub fn find_block(&mut self, code: char) -> Option<&mut SpedBlock> {
        self.blocks.iter_mut().find(|b| b.code == code)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for block in &self.blocks {
            for record in &block.records {
                out.push('|');
                out.push_str(&record.join("|"));
                out.push_str("|\n");
            }
        }
        out
    }
}

pub fn sped_open(
    profile: SpedProfile,
    cnpj: String,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> SpedFile {
    SpedFile::new(profile, cnpj, period_start, period_end)
}

pub fn sped_block(code: char) -> SpedBlock {
    SpedBlock {
        code,
        records: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn assemble_sped_file() {
        let mut file = sped_open(
            SpedProfile::Contabil,
            "11222333000181".into(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        );
        let mut block0 = sped_block('0');
        block0.records.push(vec![
            "0000".into(),
            file.period_start.format("%d%m%Y").to_string(),
            file.period_end.format("%d%m%Y").to_string(),
            file.cnpj.clone(),
        ]);
        file.add_block(block0);
        let out = file.render();
        assert!(out.contains("|0000|"));
    }
}
