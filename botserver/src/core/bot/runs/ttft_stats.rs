//! TTFT statistics (#1700).
//!
//! The chat stream records one `llm_metrics` row per turn with `ttft_ms`. This
//! module turns samples into the numbers the dashboard shows: count, min/max,
//! mean, p50, p95 and an hourly trend. Percentiles are nearest-rank, so they
//! are exactly one of the observed values and need no interpolation.

/// Summary of a set of first-token latencies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TtftSummary {
    pub count: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
}

/// Nearest-rank percentile over an ascending slice. Empty input returns 0;
/// `p` is clamped to 1..=100.
#[must_use]
pub fn percentile_ascending(samples: &[u64], p: u8) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let p = u64::from(p.clamp(1, 100));
    let n = samples.len() as u64;
    // ceil(p/100 * n) - 1, clamped to the last index.
    let rank = (p * n).div_ceil(100).max(1) - 1;
    let index = usize::try_from(rank.min(n - 1)).unwrap_or(0);
    samples[index]
}

/// Sorts once and summarizes.
#[must_use]
pub fn summarize(samples: &[u64]) -> TtftSummary {
    if samples.is_empty() {
        return TtftSummary::default();
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let count = sorted.len() as u64;
    let total: u128 = sorted.iter().map(|value| u128::from(*value)).sum();
    TtftSummary {
        count,
        min_ms: sorted[0],
        max_ms: sorted[sorted.len() - 1],
        avg_ms: u64::try_from(total / u128::from(count)).unwrap_or(u64::MAX),
        p50_ms: percentile_ascending(&sorted, 50),
        p95_ms: percentile_ascending(&sorted, 95),
    }
}

/// One bucket of the trend line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtftBucket {
    pub bucket: String,
    pub count: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
}

/// Groups `(bucket, ttft_ms)` rows into ascending buckets, keeping the bucket
/// labels produced by the query (`YYYY-MM-DD HH24:00`).
#[must_use]
pub fn trend(rows: &[(String, u64)]) -> Vec<TtftBucket> {
    let mut buckets: Vec<(String, Vec<u64>)> = Vec::new();
    for (label, value) in rows {
        match buckets.iter_mut().find(|(name, _)| name == label) {
            Some((_, values)) => values.push(*value),
            None => buckets.push((label.clone(), vec![*value])),
        }
    }
    buckets.sort_by(|left, right| left.0.cmp(&right.0));
    buckets
        .into_iter()
        .map(|(bucket, mut values)| {
            values.sort_unstable();
            TtftBucket {
                bucket,
                count: values.len() as u64,
                p50_ms: percentile_ascending(&values, 50),
                p95_ms: percentile_ascending(&values, 95),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_zeroed() {
        let summary = summarize(&[]);
        assert_eq!(summary, TtftSummary::default());
        assert_eq!(percentile_ascending(&[], 50), 0);
        assert!(trend(&[]).is_empty());
    }

    #[test]
    fn nearest_rank_percentiles_pick_observed_values() {
        let samples = [100, 200, 300, 400];
        assert_eq!(percentile_ascending(&samples, 50), 200);
        assert_eq!(percentile_ascending(&samples, 95), 400);
        assert_eq!(percentile_ascending(&samples, 1), 100);
    }

    #[test]
    fn summary_reports_count_range_mean_and_percentiles() {
        let summary = summarize(&[400, 100, 300, 200]);
        assert_eq!(summary.count, 4);
        assert_eq!(summary.min_ms, 100);
        assert_eq!(summary.max_ms, 400);
        assert_eq!(summary.avg_ms, 250);
        assert_eq!(summary.p50_ms, 200);
        assert_eq!(summary.p95_ms, 400);
    }

    #[test]
    fn trend_groups_by_hour_and_sorts_buckets() {
        let rows = vec![
            ("2026-11-05 09:00".to_string(), 300),
            ("2026-11-05 10:00".to_string(), 500),
            ("2026-11-05 09:00".to_string(), 100),
            ("2026-11-05 10:00".to_string(), 700),
        ];
        let buckets = trend(&rows);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].bucket, "2026-11-05 09:00");
        assert_eq!(buckets[0].count, 2);
        assert_eq!(buckets[0].p50_ms, 100);
        assert_eq!(buckets[1].bucket, "2026-11-05 10:00");
        assert_eq!(buckets[1].p95_ms, 700);
    }

    #[test]
    fn single_sample_is_its_own_p50_and_p95() {
        let summary = summarize(&[464]);
        assert_eq!(summary.count, 1);
        assert_eq!(summary.p50_ms, 464);
        assert_eq!(summary.p95_ms, 464);
    }
}
