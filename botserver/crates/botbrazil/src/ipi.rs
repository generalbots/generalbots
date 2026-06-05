//! IPI (Imposto sobre Produtos Industrializados) calculation helpers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpiScenario {
    Taxable,
    Exempt,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IpiBreakdown {
    pub scenario: IpiScenario,
    pub base_amount: f64,
    pub rate: f64,
    pub tax_amount: f64,
}

pub fn calculate_ipi(scenario: IpiScenario, base_amount: f64, rate: f64) -> IpiBreakdown {
    let tax_amount = match scenario {
        IpiScenario::Taxable => base_amount * rate,
        _ => 0.0,
    };
    IpiBreakdown {
        scenario,
        base_amount,
        rate,
        tax_amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxable_charges() {
        let result = calculate_ipi(IpiScenario::Taxable, 1000.0, 0.05);
        assert!((result.tax_amount - 50.0).abs() < 0.0001);
    }

    #[test]
    fn exempt_zero() {
        let result = calculate_ipi(IpiScenario::Exempt, 1000.0, 0.05);
        assert_eq!(result.tax_amount, 0.0);
    }
}
