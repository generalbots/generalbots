//! ICMS (Imposto sobre Circulação de Mercadorias e Serviços) computation
//! helpers for the main scenarios used by the NFe module.
//!
//! Only deterministic, in-memory math is performed. Real tax determination
//! requires the SEFAZ rules engine and is out of scope for this crate.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmsScenario {
    /// CSOSN 101 / 400 — Simples Nacional, ICMS not charged.
    SimplesNacional,
    /// CSOSN 102 — Simples Nacional, ICMS owes but is deferred.
    SimplesNacionalDeferred,
    /// CST 00 — Taxable, with credit.
    TaxableWithCredit,
    /// CST 10 — Collected by ST (substituição tributária).
    CollectedBySt,
    /// CST 20 — Taxable with reduction.
    TaxableWithReduction,
    /// CST 40, 41, 50, 60 — Exempt / non-taxable / suspended.
    Exempt,
    /// CST 51, 90 — Other.
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IcmsBreakdown {
    pub scenario: IcmsScenario,
    pub base_amount: f64,
    pub reduction_percent: f64,
    pub reduced_base: f64,
    pub rate: f64,
    pub tax_amount: f64,
}

pub fn calculate_icms(
    scenario: IcmsScenario,
    base_amount: f64,
    rate: f64,
    reduction_percent: f64,
) -> IcmsBreakdown {
    let reduction = reduction_percent.clamp(0.0, 100.0) / 100.0;
    let reduced_base = match scenario {
        IcmsScenario::SimplesNacional
        | IcmsScenario::SimplesNacionalDeferred
        | IcmsScenario::Exempt
        | IcmsScenario::CollectedBySt => 0.0,
        IcmsScenario::TaxableWithReduction => base_amount * (1.0 - reduction),
        _ => base_amount,
    };
    let tax_amount = match scenario {
        IcmsScenario::SimplesNacional
        | IcmsScenario::SimplesNacionalDeferred
        | IcmsScenario::Exempt => 0.0,
        IcmsScenario::CollectedBySt => reduced_base * rate,
        _ => reduced_base * rate,
    };
    IcmsBreakdown {
        scenario,
        base_amount,
        reduction_percent,
        reduced_base,
        rate,
        tax_amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simples_nacional_does_not_charge() {
        let result = calculate_icms(IcmsScenario::SimplesNacional, 1000.0, 0.18, 0.0);
        assert_eq!(result.tax_amount, 0.0);
    }

    #[test]
    fn taxable_with_credit() {
        let result = calculate_icms(IcmsScenario::TaxableWithCredit, 1000.0, 0.18, 0.0);
        assert!((result.tax_amount - 180.0).abs() < 0.0001);
    }

    #[test]
    fn taxable_with_reduction() {
        let result = calculate_icms(IcmsScenario::TaxableWithReduction, 1000.0, 0.18, 33.33);
        assert!(result.tax_amount > 100.0 && result.tax_amount < 130.0);
    }

    #[test]
    fn exempt_zero() {
        let result = calculate_icms(IcmsScenario::Exempt, 1000.0, 0.18, 0.0);
        assert_eq!(result.tax_amount, 0.0);
    }
}
