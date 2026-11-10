//! In-process SLO budgets (#2115).
//!
//! The hot paths below run on every turn, so a small regression multiplies by
//! the number of concurrent conversations. They need no server, no database and
//! no provider, which is why they are the only latency budgets that can gate a
//! pull request without a paid call.
//!
//! Two layers, on purpose:
//!
//! 1. This test fails fast with the measured number when a path crosses the
//!    budget in `BUDGETS`, so a developer sees the regression locally.
//! 2. The test writes a snapshot to `target/slo/hotpath.json` (override with
//!    `SLO_SNAPSHOT_OUT`) and CI runs
//!    `python3 scripts/check_slo.py --profile hotpath --snapshot <that file>`,
//!    which is the authority: the ratified targets live in `ops/slo.yaml`
//!    (profile `hotpath`) and are never duplicated as a second source of truth.
//!
//! The budgets are debug-build numbers (CI runs `cargo test`, not `--release`);
//! they are deliberately loose enough for a shared runner and tight enough to
//! catch an order-of-magnitude regression.

use std::path::{Path, PathBuf};
use std::time::Instant;

use botlib::i18n::{I18nBundle, Locale, MessageArgs};
use botlib::re_cache;

/// Ratified targets from `ops/slo.yaml` (profile `hotpath`). Keep in sync.
const BUDGETS: &[(&str, f64)] = &[
    ("hotpath.locale_parse_us", 20.0),
    ("hotpath.locale_negotiate_us", 10.0),
    ("hotpath.i18n_message_us", 60.0),
    ("hotpath.regex_cache_us", 30.0),
];

/// Samples per round and rounds per measurement: 200 x 100 = 20 000 operations,
/// which is long enough to smooth a scheduler hiccup and short enough for a PR.
const OPS_PER_ROUND: usize = 100;
const ROUNDS: usize = 200;

#[derive(Debug)]
struct Stats {
    p50: f64,
    p95: f64,
    p99: f64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let last = sorted.len() - 1;
    let index = ((p / 100.0) * last as f64).round() as usize;
    sorted[index.min(last)]
}

/// Per-operation microseconds, one sample per round of `OPS_PER_ROUND` calls.
fn measure(mut op: impl FnMut()) -> Stats {
    for _ in 0..OPS_PER_ROUND {
        op();
    }

    let mut samples = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let start = Instant::now();
        for _ in 0..OPS_PER_ROUND {
            op();
        }
        let micros = start.elapsed().as_secs_f64() * 1_000_000.0;
        samples.push(micros / OPS_PER_ROUND as f64);
    }

    samples.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));

    Stats {
        p50: percentile(&samples, 50.0),
        p95: percentile(&samples, 95.0),
        p99: percentile(&samples, 99.0),
    }
}

fn budget(id: &str) -> f64 {
    BUDGETS
        .iter()
        .find(|entry| entry.0 == id)
        .map(|entry| entry.1)
        .unwrap_or(0.0)
}

fn assert_budget(id: &str, stats: &Stats) {
    let limit = budget(id);
    println!(
        "slo_budget: {id} p50={:.2}us p95={:.2}us p99={:.2}us (budget p95 {limit:.0}us)",
        stats.p50, stats.p95, stats.p99
    );
    assert!(
        limit > 0.0,
        "{id} has no entry in BUDGETS; add one instead of measuring it silently"
    );
    assert!(
        stats.p95 <= limit,
        "{id} p95 {:.2}us exceeds the {limit:.0}us budget in ops/slo.yaml (profile hotpath)",
        stats.p95
    );
}

fn snapshot_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("SLO_SNAPSHOT_OUT") {
        return PathBuf::from(explicit);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/slo/hotpath.json")
}

/// Loads the shipped catalogue. `None` means the build has no embedded locales,
/// in which case the i18n budget is skipped instead of failing the run.
fn load_bundle() -> Option<I18nBundle> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../botlib/locales");
    let locales = path.to_string_lossy().into_owned();
    match I18nBundle::load(&locales) {
        Ok(bundle) => Some(bundle),
        Err(error) => {
            println!("slo_budget: catalogue unavailable ({error}); skipping the i18n budget");
            None
        }
    }
}

fn measure_message_lookup(bundle: &I18nBundle) -> Stats {
    let locale = Locale::new("pt-BR").expect("pt-BR parses");
    let mut args: MessageArgs = MessageArgs::new();
    args.insert("count".to_string(), "3".to_string());
    let key = bundle
        .message_ids()
        .into_iter()
        .next()
        .unwrap_or_else(|| "bot-greeting-default".to_string());

    measure(|| {
        let rendered = bundle.get_message(&locale, &key, Some(&args));
        assert!(!rendered.is_empty());
    })
}

fn write_snapshot(measured: &[(&str, Stats)]) -> Option<PathBuf> {
    let path = snapshot_path();
    let metrics: serde_json::Map<String, serde_json::Value> = measured
        .iter()
        .map(|(id, stats)| {
            (
                (*id).to_string(),
                serde_json::json!({
                    "p50": stats.p50,
                    "p95": stats.p95,
                    "p99": stats.p99,
                }),
            )
        })
        .collect();

    // `chrono` has no `clock` feature in this workspace, so the timestamp is the
    // Unix epoch in seconds; the checker only needs it for the report header.
    let measured_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let snapshot = serde_json::json!({
        "profile": "hotpath",
        "measured_at_epoch_s": measured_at,
        "window_days": 0,
        "source": "bottest/tests/slo_budget_test.rs",
        "metrics": metrics,
    });

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| eprintln!("slo_budget: cannot create {}: {error}", parent.display()))
            .ok()?;
    }
    let body = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| eprintln!("slo_budget: cannot serialise the snapshot: {error}"))
        .ok()?;
    std::fs::write(&path, format!("{body}\n"))
        .map_err(|error| eprintln!("slo_budget: cannot write {}: {error}", path.display()))
        .ok()?;
    println!("slo_budget: snapshot written to {}", path.display());
    Some(path)
}

/// The locale tag arrives from a channel, a header or the bot config, so parsing
/// it is on the first line of every request that renders a translated label.
#[test]
fn locale_parsing_stays_within_budget() {
    let stats = measure(|| {
        let parsed = Locale::new("pt-BR");
        assert!(parsed.is_some(), "pt-BR must parse");
    });
    assert_budget("hotpath.locale_parse_us", &stats);
}

/// Negotiation walks the shipped catalogue for every translated message.
#[test]
fn locale_negotiation_stays_within_budget() {
    let requested = Locale::new("pt-BR").expect("pt-BR parses");
    let fallback = Locale::default();
    let available: Vec<Locale> = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"]
        .iter()
        .filter_map(|tag| Locale::new(tag))
        .collect();
    assert!(!available.is_empty(), "the catalogue must offer locales");

    let stats = measure(|| {
        let negotiated = Locale::negotiate(&[&requested], &available, &fallback);
        assert_eq!(negotiated.language(), "pt");
    });
    assert_budget("hotpath.locale_negotiate_us", &stats);
}

/// Catalogue lookup plus placeholder interpolation, as called for every label.
#[test]
fn message_lookup_stays_within_budget() {
    let Some(bundle) = load_bundle() else {
        return;
    };
    let stats = measure_message_lookup(&bundle);
    assert_budget("hotpath.i18n_message_us", &stats);
}

/// Runtime regexes (classification, redaction) are memoised; a miss recompiles.
#[test]
fn regex_cache_lookup_stays_within_budget() {
    let pattern = "^slo-budget-[0-9]+$";
    assert!(
        re_cache::dynamic(pattern).is_some(),
        "the pattern must compile"
    );

    let stats = measure(|| {
        let compiled = re_cache::dynamic(pattern);
        assert!(compiled.is_some());
    });
    assert_budget("hotpath.regex_cache_us", &stats);
}

/// Emits the snapshot the CI checker validates. Kept as its own test so the
/// numbers are produced once, after the four budgets above have run.
#[test]
fn emits_the_hotpath_snapshot() {
    let locale = Locale::new("pt-BR").expect("pt-BR parses");
    let fallback = Locale::default();
    let available: Vec<Locale> = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"]
        .iter()
        .filter_map(|tag| Locale::new(tag))
        .collect();
    let pattern = "^slo-budget-[0-9]+$";
    let bundle = load_bundle().expect("the shipped catalogue must load for the snapshot");

    let mut measured = vec![
        (
            "hotpath.locale_parse_us",
            measure(|| {
                let parsed = Locale::new("pt-BR");
                assert!(parsed.is_some());
            }),
        ),
        (
            "hotpath.locale_negotiate_us",
            measure(|| {
                let negotiated = Locale::negotiate(&[&locale], &available, &fallback);
                assert_eq!(negotiated.language(), "pt");
            }),
        ),
        ("hotpath.i18n_message_us", measure_message_lookup(&bundle)),
        (
            "hotpath.regex_cache_us",
            measure(|| {
                let compiled = re_cache::dynamic(pattern);
                assert!(compiled.is_some());
            }),
        ),
    ];
    measured.sort_by(|left, right| left.0.cmp(right.0));

    let path = write_snapshot(&measured);
    assert!(
        path.is_some(),
        "the snapshot must be written for the CI checker"
    );
}
