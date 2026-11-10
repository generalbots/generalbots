//! Offline tests for the BASIC code scanner: pattern table, redaction,
//! severity ordering, per-file scanning and the report serializers.

use super::to_html::escape_csv;
use super::*;
use std::fs;
use std::io::Write;
use uuid::Uuid;

fn scanner() -> CodeScanner {
    CodeScanner::new(".").expect("the built-in pattern table must compile")
}

fn matches(line: &str) -> Vec<(IssueType, IssueSeverity)> {
    scanner()
        .patterns
        .iter()
        .filter(|pattern| pattern.regex.is_match(line))
        .map(|pattern| (pattern.issue_type.clone(), pattern.severity.clone()))
        .collect()
}

fn assert_matches(line: &str, issue_type: IssueType, severity: IssueSeverity) {
    let found = matches(line);
    assert!(
        found
            .iter()
            .any(|(kind, level)| *kind == issue_type && *level == severity),
        "expected {issue_type:?}/{severity:?} for {line:?}, got {found:?}"
    );
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("botcompliance_{tag}_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("temp dir must be creatable");
    dir
}

fn write_file(path: &Path, content: &str) {
    let mut file = fs::File::create(path).expect("temp file must be creatable");
    file.write_all(content.as_bytes())
        .expect("temp file must be writable");
}

#[test]
fn the_pattern_table_is_populated_and_internally_consistent() {
    let patterns = scanner().patterns;
    assert_eq!(patterns.len(), 12, "the published pattern table has 12 rules");

    let mut titles = std::collections::HashSet::new();
    for pattern in &patterns {
        assert!(
            titles.insert(pattern.title.clone()),
            "duplicate pattern title: {}",
            pattern.title
        );
        assert!(!pattern.remediation.is_empty(), "{} needs remediation", pattern.title);
        assert!(!pattern.category.is_empty(), "{} needs a category", pattern.title);
    }
}

#[test]
fn security_patterns_match_their_documented_samples() {
    let cases: &[(&str, IssueType, IssueSeverity)] = &[
        (
            r#"password = "hunter2hunter2""#,
            IssueType::PasswordInConfig,
            IssueSeverity::Critical,
        ),
        (
            r#"api_key = "abcdefgh12345678""#,
            IssueType::HardcodedSecret,
            IssueSeverity::Critical,
        ),
        (
            r#"token = "abcdefghijklmnopqrstuvwxyz""#,
            IssueType::HardcodedSecret,
            IssueSeverity::High,
        ),
        (
            "AKIAIOSFODNN7EXAMPLE",
            IssueType::HardcodedSecret,
            IssueSeverity::Critical,
        ),
        (
            "-----BEGIN RSA PRIVATE KEY-----",
            IssueType::HardcodedSecret,
            IssueSeverity::Critical,
        ),
        (
            "postgres://admin:s3cret@db:5432/app",
            IssueType::HardcodedSecret,
            IssueSeverity::Critical,
        ),
        (
            "POST TO INSTAGRAM user, pass",
            IssueType::InsecurePattern,
            IssueSeverity::High,
        ),
    ];
    for (line, issue_type, severity) in cases {
        assert_matches(line, issue_type.clone(), severity.clone());
    }
}

#[test]
fn code_quality_patterns_match_their_documented_samples() {
    let cases: &[(&str, IssueType, IssueSeverity)] = &[
        (
            "IF response = input THEN",
            IssueType::DeprecatedIfInput,
            IssueSeverity::Medium,
        ),
        (
            r#"USE_KB "docs""#,
            IssueType::UnderscoreInKeyword,
            IssueSeverity::Low,
        ),
        (
            "SELECT * FROM users",
            IssueType::FragileCode,
            IssueSeverity::Medium,
        ),
        (r#"EVAL("1+1")"#, IssueType::FragileCode, IssueSeverity::High),
    ];
    for (line, issue_type, severity) in cases {
        assert_matches(line, issue_type.clone(), severity.clone());
    }
}

#[test]
fn ordinary_basic_lines_are_not_flagged() {
    for line in [
        r#"TALK "Hello""#,
        "x = 1",
        "IF x > 1 THEN",
        "SAVE customer",
        r#"GET FROM inventory WHERE quantity < 10"#,
        "",
    ] {
        assert!(
            matches(line).is_empty(),
            "{line:?} must not be flagged, got {:?}",
            matches(line)
        );
    }
}

#[test]
fn severity_order_runs_from_info_to_critical() {
    let ladder = [
        IssueSeverity::Info,
        IssueSeverity::Low,
        IssueSeverity::Medium,
        IssueSeverity::High,
        IssueSeverity::Critical,
    ];
    for window in ladder.windows(2) {
        assert!(
            window[0] < window[1],
            "{:?} must sort before {:?}",
            window[0],
            window[1]
        );
    }
    let labels: Vec<String> = ladder.iter().map(ToString::to_string).collect();
    assert_eq!(labels, ["info", "low", "medium", "high", "critical"]);
}

#[test]
fn issue_types_have_human_readable_labels() {
    let pairs = [
        (IssueType::PasswordInConfig, "Password in Config"),
        (IssueType::HardcodedSecret, "Hardcoded Secret"),
        (IssueType::DeprecatedKeyword, "Deprecated Keyword"),
        (IssueType::FragileCode, "Fragile Code"),
        (IssueType::ConfigurationIssue, "Configuration Issue"),
        (IssueType::UnderscoreInKeyword, "Underscore in Keyword"),
        (IssueType::MissingVault, "Missing Vault Config"),
        (IssueType::InsecurePattern, "Insecure Pattern"),
        (IssueType::DeprecatedIfInput, "Deprecated IF...input Pattern"),
    ];
    for (issue_type, label) in pairs {
        assert_eq!(issue_type.to_string(), label);
    }
}

#[test]
fn redaction_hides_quoted_values_and_aws_keys() {
    let redacted = CodeScanner::redact_sensitive(r#"password = "supersecret""#);
    assert_eq!(redacted, r#"password = "***REDACTED***""#);
    assert!(!redacted.contains("supersecret"));

    let redacted = CodeScanner::redact_sensitive("key=AKIAIOSFODNN7EXAMPLE");
    assert_eq!(redacted, "key=AKIA***REDACTED***");
    assert!(!redacted.contains("IOSFODNN7EXAMPLE"));

    let short = r#"flag = "abc""#;
    assert_eq!(
        CodeScanner::redact_sensitive(short),
        short,
        "short literals stay readable"
    );
}

#[test]
fn csv_escaping_quotes_only_when_needed() {
    let cases = [
        ("plain", "plain"),
        ("with,comma", "\"with,comma\""),
        ("with \"quote\"", "\"with \"\"quote\"\"\""),
        ("two\nlines", "\"two\nlines\""),
        ("", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(escape_csv(input), expected, "escaping {input:?}");
    }
}

#[tokio::test]
async fn scanning_a_file_skips_comments_and_numbers_lines() {
    let dir = temp_dir("scan_file");
    let path = dir.join("start.bas");
    write_file(
        &path,
        "REM password = \"commentedsecret\"\n\
         ' token = \"alsocommented\"\n\
         // EVAL(\"nope\")\n\
         password = \"realpassword\"\n\
         TALK \"hello\"\n",
    );

    let issues = scanner()
        .scan_file(&path)
        .await
        .expect("the temp file must be scannable");

    assert_eq!(issues.len(), 1, "only the live line is flagged: {issues:?}");
    let issue = &issues[0];
    assert_eq!(issue.issue_type, IssueType::PasswordInConfig);
    assert_eq!(issue.severity, IssueSeverity::Critical);
    assert_eq!(issue.line_number, Some(4));
    let snippet = issue.code_snippet.as_deref().expect("snippet is recorded");
    assert!(!snippet.contains("realpassword"), "snippet must be redacted");
    assert!(snippet.contains("REDACTED"));

    fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn scanning_a_bot_counts_files_and_reports_a_missing_vault() {
    let dir = temp_dir("scan_bot");
    let bot = dir.join("demo.gbdialog");
    fs::create_dir_all(&bot).expect("bot dir");
    write_file(&bot.join("start.bas"), "password = \"hunter2hunter2\"\n");
    write_file(&bot.join("data.csv"), "id,name\n1,ok\n");
    write_file(&bot.join("config.csv"), "name=demo\n");

    let result = scanner()
        .scan_bot(&bot)
        .await
        .expect("the bot directory must be scannable");

    assert_eq!(result.bot_name, "demo.gbdialog");
    assert_eq!(
        result.files_scanned, 3,
        "start.bas, data.csv and config.csv are all scanned"
    );
    assert_eq!(result.stats.critical, 1);
    assert_eq!(result.stats.info, 1, "the missing Vault config is info level");
    assert_eq!(result.stats.total, 2);
    assert_eq!(
        result.issues[0].severity,
        IssueSeverity::Critical,
        "issues are sorted most severe first"
    );
    assert!(result
        .issues
        .iter()
        .any(|issue| issue.issue_type == IssueType::MissingVault));

    write_file(&bot.join("config.csv"), "vault_addr=https://vault:8200\n");
    let configured = scanner()
        .scan_bot(&bot)
        .await
        .expect("the bot directory must be scannable");
    assert!(
        !configured
            .issues
            .iter()
            .any(|issue| issue.issue_type == IssueType::MissingVault),
        "a vault_addr entry clears the missing Vault finding"
    );

    fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn scan_all_walks_templates_and_work_trees() {
    let dir = temp_dir("scan_all");
    let template_bot = dir.join("templates").join("alpha.gbdialog");
    let work_bot = dir.join("work").join("beta.gbdialog");
    fs::create_dir_all(&template_bot).expect("template bot dir");
    fs::create_dir_all(&work_bot).expect("work bot dir");
    write_file(&template_bot.join("start.bas"), "EVAL(\"x\")\n");
    write_file(&work_bot.join("start.bas"), "EVAL(\"y\")\n");

    let result = CodeScanner::new(&dir)
        .expect("scanner builds")
        .scan_all()
        .await
        .expect("the tree must be scannable");

    assert_eq!(result.bots_scanned, 2, "both trees are walked");
    assert_eq!(result.total_files, 2);
    assert_eq!(result.stats.high, 2);
    assert_eq!(result.stats.total, 2);
    assert!(result
        .bot_results
        .iter()
        .any(|bot| bot.bot_name == "alpha.gbdialog"));
    assert!(result
        .bot_results
        .iter()
        .any(|bot| bot.bot_name == "beta.gbdialog"));

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn csv_reports_use_the_published_header_and_escape_fields() {
    let dir = temp_dir("csv");
    let issue = CodeIssue {
        id: "id-1".to_string(),
        severity: IssueSeverity::Critical,
        issue_type: IssueType::PasswordInConfig,
        title: "Hardcoded, Password".to_string(),
        description: "line one\nline two".to_string(),
        file_path: "bot.gbdialog/start.bas".to_string(),
        line_number: Some(7),
        code_snippet: None,
        remediation: "Use Vault".to_string(),
        category: "Security".to_string(),
        detected_at: Utc::now(),
    };
    let result = ComplianceScanResult {
        scanned_at: Utc::now(),
        duration_ms: 3,
        bots_scanned: 1,
        total_files: 1,
        stats: ScanStats {
            critical: 1,
            total: 1,
            ..ScanStats::default()
        },
        bot_results: vec![BotScanResult {
            bot_id: "bot-1".to_string(),
            bot_name: "bot".to_string(),
            scanned_at: Utc::now(),
            files_scanned: 1,
            issues: vec![issue],
            stats: ScanStats {
                critical: 1,
                total: 1,
                ..ScanStats::default()
            },
        }],
    };

    let csv = ComplianceReporter::to_csv(&result);
    let header = csv.lines().next().expect("csv has a header");
    assert_eq!(
        header,
        "Severity,Type,Category,File,Line,Title,Description,Remediation"
    );
    assert!(csv.contains("\"Hardcoded, Password\""));
    assert!(csv.contains("\"line one\nline two\""));

    let html = ComplianceReporter::to_html(&result);
    assert!(html.contains("class='critical'"));
    assert!(html.contains("Password in Config"));
    assert!(html.contains("<td>7</td>"));

    let json = ComplianceReporter::to_json(&result).expect("report serializes");
    assert!(json.contains("\"severity\": \"critical\""));

    fs::remove_dir_all(&dir).ok();
}
