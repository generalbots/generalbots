use super::*;

/// Catalogue key of the localized name of each quota metric (#2094).
fn metric_key(metric: UsageMetric) -> &'static str {
    match metric {
        UsageMetric::Messages => "email-metric-messages",
        UsageMetric::StorageBytes => "email-metric-storage",
        UsageMetric::ApiCalls => "email-metric-api-calls",
        UsageMetric::Bots => "email-metric-bots",
        UsageMetric::Users => "email-metric-users",
        UsageMetric::KbDocuments => "email-metric-kb-documents",
        UsageMetric::Apps => "email-metric-apps",
        UsageMetric::EmailsPerDay => "email-metric-emails-per-day",
        UsageMetric::BuildsPerDay => "email-metric-builds-per-day",
        UsageMetric::UploadsPerHour => "email-metric-uploads-per-hour",
    }
}

impl QuotaManager {
    pub(super) fn alert_message(&self, metric: UsageMetric, threshold: f64, current: f64) -> String {
        let metric_name = botlib::i18n::translate(&self.locale, metric_key(metric), &[]);

        if current >= 100.0 {
            botlib::i18n::translate(
                &self.locale,
                "email-quota-limit-reached",
                &[("metric", &metric_name)],
            )
        } else {
            botlib::i18n::translate(
                &self.locale,
                "email-quota-threshold",
                &[
                    ("metric", &metric_name),
                    ("current", &(current as u32).to_string()),
                    ("threshold", &(threshold as u32).to_string()),
                ],
            )
        }
    }
}
