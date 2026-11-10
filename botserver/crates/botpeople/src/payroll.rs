use chrono::NaiveDate;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::models::{Payroll, PayrollStatus};

type Storage = Arc<Mutex<HashMap<Uuid, Payroll>>>;

#[derive(Clone)]
pub struct PayrollService {
    storage: Storage,
}

impl PayrollService {
    pub fn new() -> Self {
        PayrollService {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn process_month(
        &self,
        employee_id: Uuid,
        gross_pay: f64,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Payroll, String> {
        let id = Uuid::new_v4();
        let deductions = self.calculate_deductions(gross_pay);
        let net_pay = gross_pay - deductions;
        let payroll = Payroll {
            id,
            employee_id,
            period_start,
            period_end,
            gross_pay,
            deductions,
            net_pay,
            status: PayrollStatus::Completed,
        };
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.insert(id, payroll.clone());
        Ok(payroll)
    }

    fn calculate_deductions(&self, gross_pay: f64) -> f64 {
        let inss = (gross_pay * 0.08).min(1000.0);
        let irrf = if gross_pay > 5000.0 {
            (gross_pay - 5000.0) * 0.15
        } else if gross_pay > 3000.0 {
            (gross_pay - 3000.0) * 0.075
        } else {
            0.0
        };
        (inss + irrf).round()
    }

    pub fn get_payslip(&self, id: Uuid) -> Result<Payroll, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        store.get(&id).cloned().ok_or_else(|| format!("Payroll not found: {id}"))
    }

    pub fn list_by_employee(&self, employee_id: Uuid) -> Result<Vec<Payroll>, String> {
        let store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut records: Vec<Payroll> = store
            .values()
            .filter(|p| p.employee_id == employee_id)
            .cloned()
            .collect();
        records.sort_by_key(|a| std::cmp::Reverse(a.period_start));
        Ok(records)
    }

    pub fn mark_paid(&self, id: Uuid) -> Result<Payroll, String> {
        let mut store = self.storage.lock().map_err(|e| format!("Lock error: {e}"))?;
        let payroll = store.get_mut(&id).ok_or_else(|| format!("Payroll not found: {id}"))?;
        payroll.status = PayrollStatus::Paid;
        Ok(payroll.clone())
    }

    pub fn generate_payslip_text(&self, id: Uuid) -> Result<String, String> {
        let payroll = self.get_payslip(id)?;
        let text = format!(
            "Payslip\nPeriod: {} - {}\nGross Pay: ${:.2}\nDeductions: ${:.2}\nNet Pay: ${:.2}\nStatus: {}",
            payroll.period_start,
            payroll.period_end,
            payroll.gross_pay,
            payroll.deductions,
            payroll.net_pay,
            payroll.status.as_str(),
        );
        Ok(text)
    }
}

impl Default for PayrollService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod payroll_tests {
    use super::*;

    fn period() -> (NaiveDate, NaiveDate) {
        (
            NaiveDate::from_ymd_opt(2026, 11, 1).expect("start"),
            NaiveDate::from_ymd_opt(2026, 11, 30).expect("end"),
        )
    }

    #[test]
    fn a_run_completes_with_net_equal_to_gross_minus_deductions() {
        let service = PayrollService::new();
        let (start, end) = period();
        let payroll = service
            .process_month(Uuid::new_v4(), 4000.0, start, end)
            .expect("payroll");

        assert_eq!(payroll.status, PayrollStatus::Completed);
        assert_eq!(payroll.gross_pay, 4000.0);
        assert!((payroll.net_pay - (4000.0 - payroll.deductions)).abs() < 0.001);
        assert_eq!(payroll.period_start, start);
        assert_eq!(payroll.period_end, end);
    }

    #[test]
    fn deductions_follow_the_documented_brackets() {
        let service = PayrollService::new();
        let (start, end) = period();

        // Below 3000: only the 8% INSS.
        let low = service
            .process_month(Uuid::new_v4(), 2000.0, start, end)
            .expect("low");
        assert_eq!(low.deductions, (2000.0 * 0.08_f64).round());

        // 3000..5000: INSS plus 7.5% above 3000.
        let mid = service
            .process_month(Uuid::new_v4(), 4000.0, start, end)
            .expect("mid");
        let expected_mid = (4000.0 * 0.08_f64 + 1000.0 * 0.075_f64).round();
        assert_eq!(mid.deductions, expected_mid);

        // Above 5000: INSS plus 15% above 5000.
        let high = service
            .process_month(Uuid::new_v4(), 6000.0, start, end)
            .expect("high");
        let expected_high = (6000.0 * 0.08_f64 + 1000.0 * 0.15_f64).round();
        assert_eq!(high.deductions, expected_high);
    }

    #[test]
    fn the_inss_contribution_is_capped() {
        let service = PayrollService::new();
        let (start, end) = period();
        // 8% of 20 000 is 1600, above the 1000 cap.
        let payroll = service
            .process_month(Uuid::new_v4(), 20_000.0, start, end)
            .expect("payroll");
        let irrf = (20_000.0 - 5000.0) * 0.15;
        assert_eq!(payroll.deductions, (1000.0_f64 + irrf).round());
    }

    #[test]
    fn an_unknown_payslip_is_reported() {
        let service = PayrollService::new();
        assert!(service.get_payslip(Uuid::new_v4()).is_err());
        assert!(service.mark_paid(Uuid::new_v4()).is_err());
        assert!(service.generate_payslip_text(Uuid::new_v4()).is_err());
    }

    #[test]
    fn payslips_are_listed_newest_period_first() {
        let service = PayrollService::new();
        let employee = Uuid::new_v4();
        let (start, end) = period();

        service
            .process_month(employee, 1000.0, start, end)
            .expect("november");
        service
            .process_month(
                employee,
                1000.0,
                NaiveDate::from_ymd_opt(2026, 10, 1).expect("start"),
                NaiveDate::from_ymd_opt(2026, 10, 31).expect("end"),
            )
            .expect("october");

        let list = service.list_by_employee(employee).expect("list");
        assert_eq!(list.len(), 2);
        assert!(list[0].period_start > list[1].period_start);
    }

    #[test]
    fn mark_paid_updates_only_the_status() {
        let service = PayrollService::new();
        let (start, end) = period();
        let payroll = service
            .process_month(Uuid::new_v4(), 3000.0, start, end)
            .expect("payroll");

        let paid = service.mark_paid(payroll.id).expect("paid");
        assert_eq!(paid.status, PayrollStatus::Paid);
        assert_eq!(paid.gross_pay, payroll.gross_pay);
        assert_eq!(paid.net_pay, payroll.net_pay);
    }

    #[test]
    fn the_payslip_text_reports_every_amount() {
        let service = PayrollService::new();
        let (start, end) = period();
        let payroll = service
            .process_month(Uuid::new_v4(), 5000.0, start, end)
            .expect("payroll");

        let text = service.generate_payslip_text(payroll.id).expect("text");
        assert!(text.contains("Payslip"));
        assert!(text.contains("2026-11-01 - 2026-11-30"));
        assert!(text.contains("Gross Pay: $5000.00"));
        assert!(text.contains(&format!("Deductions: ${:.2}", payroll.deductions)));
        assert!(text.contains(&format!("Net Pay: ${:.2}", payroll.net_pay)));
    }

    #[test]
    fn records_are_scoped_per_employee() {
        let service = PayrollService::new();
        let (start, end) = period();
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        service.process_month(first, 1000.0, start, end).expect("a");
        service.process_month(second, 1000.0, start, end).expect("b");
        assert_eq!(service.list_by_employee(first).expect("a").len(), 1);
        assert_eq!(service.list_by_employee(second).expect("b").len(), 1);
    }
}
