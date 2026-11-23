//! `gb audit <product> <version>` — scriptable advisory check (#2723).
//!
//! Queries the public advisory API (or a local bundle with `--offline`) and
//! exits 1 when the checked version is affected, 0 when it is not. Exit 2 is
//! reserved for usage errors, so scripts can distinguish "vulnerable" from
//! "could not check".

use anyhow::{anyhow, Context, Result};
use botlib::security::advisory::{Advisory, AdvisoryQuery, AdvisoryStore};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{env, fs, path::Path, time::Duration};

const DEFAULT_API: &str = "http://localhost:8080/api/advisories";

#[derive(Debug, Deserialize)]
struct AdvisoryResponse {
    advisories: Vec<Advisory>,
}

struct Options {
    product: String,
    version: String,
    api: String,
    offline: Option<String>,
    json_output: bool,
    lang: String,
}

fn usage() {
    eprintln!("Usage: gb audit <product> <version> [--api <url>] [--offline <bundle.json>] [--json] [--lang en|pt-BR]");
}

fn pick<'a>(lang: &str, en: &'a str, pt: &'a str) -> &'a str {
    if lang.to_ascii_lowercase().starts_with("pt") {
        pt
    } else {
        en
    }
}

fn parse_options(args: &[String]) -> Result<Option<Options>> {
    let product = match args.get(2) {
        Some(value) if !value.starts_with('-') => value.clone(),
        _ => {
            usage();
            return Ok(None);
        }
    };
    let version = match args.get(3) {
        Some(value) if !value.starts_with('-') => value.clone(),
        _ => {
            usage();
            return Ok(None);
        }
    };
    let api = env::var("GB_ADVISORY_API").unwrap_or_else(|_| DEFAULT_API.to_string());
    let lang = env::var("LANG").unwrap_or_else(|_| "en".to_string());
    let mut options = Options {
        product,
        version,
        api,
        offline: None,
        json_output: false,
        lang,
    };
    let mut index = 4;
    while index < args.len() {
        match args[index].as_str() {
            "--api" => {
                options.api = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--api requires a URL"))?
                    .clone();
                index += 2;
            }
            "--offline" => {
                options.offline = Some(
                    args.get(index + 1)
                        .ok_or_else(|| anyhow!("--offline requires a bundle path"))?
                        .clone(),
                );
                index += 2;
            }
            "--lang" => {
                options.lang = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--lang requires en or pt-BR"))?
                    .clone();
                index += 2;
            }
            "--json" => {
                options.json_output = true;
                index += 1;
            }
            "--help" | "-h" => {
                usage();
                return Ok(None);
            }
            other => return Err(anyhow!("unknown option: {other}")),
        }
    }
    Ok(Some(options))
}

async fn fetch_advisories(options: &Options) -> Result<Vec<Advisory>> {
    if let Some(path) = &options.offline {
        let raw = fs::read_to_string(Path::new(path))
            .with_context(|| format!("cannot read offline advisory bundle: {path}"))?;
        let parsed: Value = serde_json::from_str(&raw).context("invalid offline bundle JSON")?;
        let records = parsed.get("advisories").unwrap_or(&parsed).clone();
        let advisories: Vec<Advisory> = serde_json::from_value(records)
            .context("offline bundle does not contain canonical advisories")?;
        return Ok(advisories);
    }

    let mut url = reqwest::Url::parse(&options.api)
        .with_context(|| format!("invalid advisory API URL: {}", options.api))?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("product", &options.product);
        query.append_pair("version", &options.version);
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("cannot build HTTP client")?;
    let response = client
        .get(url)
        .send()
        .await
        .context("cannot reach advisory API")?
        .error_for_status()
        .context("advisory API returned an error")?;
    let payload: AdvisoryResponse = response.json().await.context("invalid advisory API JSON")?;
    Ok(payload.advisories)
}

fn recommended_upgrade(matches: &[&Advisory]) -> String {
    let mut versions: Vec<&str> = matches
        .iter()
        .map(|advisory| advisory.remediation.recommended_version.as_str())
        .collect();
    versions.sort_unstable();
    versions.dedup();
    versions.join(", ")
}

fn print_text(options: &Options, matches: &[&Advisory]) {
    let affected = !matches.is_empty();
    let status = if affected {
        pick(&options.lang, "AFFECTED", "AFETADO")
    } else {
        pick(&options.lang, "NOT AFFECTED", "NAO AFETADO")
    };
    println!(
        "{}",
        pick(
            &options.lang,
            "General Bots advisory audit",
            "Auditoria de avisos General Bots"
        )
    );
    println!(
        "{}: {}",
        pick(&options.lang, "Product", "Produto"),
        options.product
    );
    println!(
        "{}: {}",
        pick(&options.lang, "Version", "Versao"),
        options.version
    );
    println!("{}: {}", pick(&options.lang, "Status", "Estado"), status);
    if affected {
        println!("{}:", pick(&options.lang, "Advisories", "Avisos"));
        for advisory in matches {
            println!(
                "  - {} [{}] {}",
                advisory.id,
                advisory.severity.as_str(),
                advisory.title
            );
        }
        println!(
            "{}: {}",
            pick(
                &options.lang,
                "Recommended upgrade",
                "Atualizacao recomendada"
            ),
            recommended_upgrade(matches)
        );
    } else {
        println!(
            "{}",
            pick(
                &options.lang,
                "No advisory affects this version.",
                "Nenhum aviso afeta esta versao."
            )
        );
    }
}

fn print_json(options: &Options, matches: &[&Advisory]) {
    let payload = json!({
        "product": options.product,
        "version": options.version,
        "affected": !matches.is_empty(),
        "advisories": matches.iter().map(|advisory| json!({
            "id": advisory.id,
            "severity": advisory.severity.as_str(),
            "title": advisory.title,
            "recommended_version": advisory.remediation.recommended_version,
        })).collect::<Vec<Value>>(),
        "recommended_upgrade": recommended_upgrade(matches),
        "exit_code": if matches.is_empty() { 0 } else { 1 },
    });
    println!("{payload}");
}

/// Returns the process exit code: 0 clean, 1 affected, 2 usage error.
pub async fn run(args: &[String]) -> Result<i32> {
    let options = match parse_options(args)? {
        Some(options) => options,
        None => return Ok(2),
    };
    let advisories = fetch_advisories(&options).await?;
    let store = AdvisoryStore::new(advisories).context("invalid advisory bundle")?;
    let matches = store.query(&AdvisoryQuery {
        product: Some(options.product.clone()),
        version: Some(options.version.clone()),
        ..AdvisoryQuery::default()
    });
    if options.json_output {
        print_json(&options, &matches);
    } else {
        print_text(&options, &matches);
    }
    Ok(if matches.is_empty() { 0 } else { 1 })
}
