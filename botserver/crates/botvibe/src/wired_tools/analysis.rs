//! Analysis group of wired tools (Issue #796): market data, sentiment,
//! report generation and anomaly detection. All pure algorithms; the market
//! feed optionally hits the public Stooq CSV endpoint.

use super::{err, handler, number_array, ok, opt_str, require_str};
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::VibeState;
use serde_json::{json, Value};

/// Positive/negative lexicon (EN + PT) for `analyze_sentiment`.
const POSITIVE: &[&str] = &[
    "bom", "boa", "ótimo", "otimo", "excelente", "cresceu", "crescer", "lucro", "ganho",
    "recomendo", "confiável", "confiavel", "vantajoso", "alta", "ganhando",
    "good", "great", "excellent", "profit", "gain", "grew", "growth", "recommend",
    "reliable", "bullish", "up",
];
const NEGATIVE: &[&str] = &[
    "ruim", "péssimo", "pessimo", "queda", "perda", "perdas", "prejuízo", "prejuizo",
    "fraco", "arriscado", "não recomendo", "nao recomendo", "falhou", "baixa",
    "bad", "poor", "loss", "losses", "fell", "drop", "decline", "risky", "bearish",
    "down", "worry",
];

/// Keyword sentiment score in [-1, 1] with a confidence based on hits.
fn sentiment_score(text: &str) -> (f64, f64) {
    let lower = text.to_lowercase();
    let positive_hits = POSITIVE.iter().filter(|w| lower.contains(**w)).count();
    let negative_hits = NEGATIVE.iter().filter(|w| lower.contains(**w)).count();
    let total = positive_hits + negative_hits;
    if total == 0 {
        return (0.0, 0.0);
    }
    let score = (positive_hits as f64 - negative_hits as f64) / total as f64;
    let confidence = (total as f64 / 3.0).min(1.0);
    (score, confidence)
}

/// z-score anomaly detection over a series; `threshold` = |z| cutoff.
fn detect_anomalies(series: &[f64], threshold: f64) -> (f64, f64, Vec<Value>) {
    let n = series.len() as f64;
    let mean = series.iter().sum::<f64>() / n;
    let variance = series.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    let anomalies = series
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            if std == 0.0 {
                return None;
            }
            let z = (v - mean) / std;
            if z.abs() > threshold {
                Some(json!({ "index": i, "value": v, "z_score": (z * 100.0).round() / 100.0 }))
            } else {
                None
            }
        })
        .collect();
    (mean, std, anomalies)
}

/// Renders a markdown financial report from title + metrics + summary.
fn render_report(title: &str, metrics: &Value, summary: &str) -> String {
    let mut out = format!("# {title}\n\n");
    if let Some(rows) = metrics.as_array() {
        out.push_str("| Metric | Value |\n|---|---|\n");
        for row in rows {
            let label = row.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let value = row.get("value").map(|v| v.to_string()).unwrap_or_default();
            out.push_str(&format!("| {label} | {value} |\n"));
        }
    }
    if !summary.is_empty() {
        out.push_str(&format!("\n## Summary\n\n{summary}\n"));
    }
    out
}

/// Parses the Stooq CSV quote payload into JSON rows.
fn parse_stooq_csv(csv: &str) -> Vec<Value> {
    csv.lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 7 {
                return None;
            }
            let close = cols[6].parse::<f64>().ok()?;
            Some(json!({
                "symbol": cols[0],
                "date": cols[1],
                "time": cols[2],
                "open": cols[3].parse::<f64>().unwrap_or(0.0),
                "high": cols[4].parse::<f64>().unwrap_or(0.0),
                "low": cols[5].parse::<f64>().unwrap_or(0.0),
                "close": close,
                "volume": cols[7].parse::<u64>().unwrap_or(0),
            }))
        })
        .collect()
}

/// `fetch_market_data` — real-time market data from the public Stooq CSV
/// endpoint (no API key); `csv` argument bypasses the network for callers
/// that already hold a payload.
fn fetch_market_data() -> ToolHandler {
    handler(|args, _state: &dyn VibeState| async move {
        let symbol = match require_str(&args, "symbol") {
            Ok(s) => s.trim().to_uppercase(),
            Err(e) => return err(e),
        };
        if let Some(csv) = args.get("csv").and_then(|v| v.as_str()) {
            let rows = parse_stooq_csv(csv);
            return ok(json!({ "symbol": symbol, "rows": rows, "source": "provided" }));
        }
        let url = format!("https://stooq.com/q/l/?s={symbol}&f=sd2t2ohlcv&h&e=csv");
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
        {
            Ok(c) => c,
            Err(e) => return err(format!("http client build failed: {e}")),
        };
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let text = resp.text().await.unwrap_or_default();
                let rows = parse_stooq_csv(&text);
                if rows.is_empty() {
                    return ok(json!({
                        "symbol": symbol,
                        "rows": [],
                        "note": "No quotes returned (symbol may be invalid)"
                    }));
                }
                ok(json!({ "symbol": symbol, "rows": rows, "source": "stooq" }))
            }
            Ok(resp) => err(format!("market endpoint returned status {}", resp.status())),
            Err(e) => err(format!("market request failed: {e}")),
        }
    })
}

/// `analyze_sentiment` — keyword sentiment analysis over the provided text.
fn analyze_sentiment() -> ToolHandler {
    handler(|args, _state: &dyn VibeState| async move {
        let text = match require_str(&args, "text") {
            Ok(t) => t.to_string(),
            Err(e) => return err(e),
        };
        let (score, confidence) = sentiment_score(&text);
        let label = if score > 0.15 {
            "positive"
        } else if score < -0.15 {
            "negative"
        } else {
            "neutral"
        };
        ok(json!({
            "label": label,
            "score": (score * 100.0).round() / 100.0,
            "confidence": (confidence * 100.0).round() / 100.0,
        }))
    })
}

/// `generate_report` — builds a markdown report from structured metrics.
fn generate_report() -> ToolHandler {
    handler(|args, _state: &dyn VibeState| async move {
        let title = match require_str(&args, "title") {
            Ok(t) => t.to_string(),
            Err(e) => return err(e),
        };
        let metrics = args.get("metrics").cloned().unwrap_or_else(|| json!([]));
        let summary = opt_str(&args, "summary", "");
        let markdown = render_report(&title, &metrics, &summary);
        ok(json!({ "title": title, "markdown": markdown, "metrics": metrics }))
    })
}

/// `detect_anomalies` — z-score anomaly detection over a time series.
fn detect_anomalies_tool() -> ToolHandler {
    handler(|args, _state: &dyn VibeState| async move {
        let series = match number_array(&args, "series") {
            Ok(s) => s,
            Err(e) => return err(e),
        };
        if series.is_empty() {
            return err("series must not be empty".into());
        }
        let threshold = args.get("threshold").and_then(|v| v.as_f64()).unwrap_or(2.0);
        let (mean, std, anomalies) = detect_anomalies(&series, threshold);
        ok(json!({
            "mean": (mean * 100.0).round() / 100.0,
            "std": (std * 100.0).round() / 100.0,
            "anomaly_count": anomalies.len(),
            "anomalies": anomalies,
        }))
    })
}

/// Registration triplets for the analysis group.
pub fn analysis_tools() -> Vec<(String, ToolSchema, ToolHandler)> {
    use crate::types::VibeUseCase;
    let cases = vec![VibeUseCase::FinancialAnalysis];
    vec![
        ("fetch_market_data".into(),
            ToolSchema::new("fetch_market_data", "Real-time market data (Stooq CSV, no key)")
                .with_parameters(json!({
                    "type": "object",
                    "properties": {
                        "symbol": {"type": "string", "description": "Ticker symbol (e.g. AAPL, PETR4.SA)"},
                        "csv": {"type": "string", "description": "Optional pre-fetched Stooq CSV payload (bypasses network)"}
                    },
                    "required": ["symbol"]
                }))
                .with_use_cases(cases.clone()),
            fetch_market_data()),
        ("analyze_sentiment".into(),
            ToolSchema::new("analyze_sentiment", "Market sentiment analysis (EN/PT keywords)")
                .with_parameters(json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to analyze"}
                    },
                    "required": ["text"]
                }))
                .with_use_cases(cases.clone()),
            analyze_sentiment()),
        ("generate_report".into(),
            ToolSchema::new("generate_report", "Generate a financial markdown report")
                .with_parameters(json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string", "description": "Report title"},
                        "metrics": {"type": "array", "items": {"type": "object"}, "description": "[{label, value}] rows"},
                        "summary": {"type": "string", "description": "Executive summary"}
                    },
                    "required": ["title"]
                }))
                .with_use_cases(cases.clone()),
            generate_report()),
        ("detect_anomalies".into(),
            ToolSchema::new("detect_anomalies", "Time series anomaly detection (z-score)")
                .with_parameters(json!({
                    "type": "object",
                    "properties": {
                        "series": {"type": "array", "items": {"type": "number"}, "description": "Numeric time series"},
                        "threshold": {"type": "number", "description": "|z| cutoff (default 2.0)"}
                    },
                    "required": ["series"]
                }))
                .with_use_cases(cases),
            detect_anomalies_tool()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentiment_scores_positive_negative_neutral() {
        assert!(sentiment_score("great earnings and strong growth").0 > 0.0);
        assert!(sentiment_score("perdas e queda forte e arriscado").0 < 0.0);
        assert_eq!(sentiment_score("qwerty xyz").0, 0.0);
    }

    #[test]
    fn anomalies_detected_with_z_score() {
        let series = vec![10.0, 10.1, 9.9, 10.0, 25.0, 10.0, 9.8];
        let (_, _, anomalies) = detect_anomalies(&series, 2.0);
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0]["index"], 4);
        assert_eq!(anomalies[0]["value"], 25.0);
    }

    #[test]
    fn flat_series_has_no_anomalies() {
        let (_, _, anomalies) = detect_anomalies(&[5.0, 5.0, 5.0], 2.0);
        assert!(anomalies.is_empty());
    }

    #[test]
    fn report_renders_markdown_table() {
        let md = render_report("Q2", &json!([{"label": "Revenue", "value": 1000}]), "Solid quarter");
        assert!(md.contains("# Q2"));
        assert!(md.contains("| Revenue | 1000 |"));
        assert!(md.contains("## Summary"));
    }

    #[test]
    fn stooq_csv_parsed_into_rows() {
        let csv = "Symbol,Date,Time,Open,High,Low,Close,Volume\nAAPL,2026-01-02,16:30,185.6,188.3,184.5,187.0,59706200\n";
        let rows = parse_stooq_csv(csv);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["symbol"], "AAPL");
        assert_eq!(rows[0]["close"], 187.0);
        assert_eq!(rows[0]["volume"], 59706200u64);
    }
}
