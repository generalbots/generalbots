use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInspectionResult {
    pub allowed: bool,
    pub blocked_patterns: Vec<BlockedPattern>,
    pub risk_level: RiskLevel,
    pub script_id: Option<String>,
    pub inspector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPattern {
    pub pattern: String,
    pub description: String,
    pub severity: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Suspicious,
    Blocked,
}

impl RiskLevel {
    pub fn is_blocked(&self) -> bool {
        matches!(self, RiskLevel::Blocked)
    }
}

static DANGEROUS_PATTERNS: &[(&str, &str, RiskLevel)] = &[
    ("Command::new", "Shell command execution via Command::new", RiskLevel::Blocked),
    ("std::process::Command", "Shell command execution", RiskLevel::Blocked),
    ("std::fs", "Filesystem access outside sandbox", RiskLevel::Suspicious),
    ("std::net", "Network access outside sandbox", RiskLevel::Suspicious),
    ("eval(", "Dynamic code evaluation", RiskLevel::Blocked),
    ("std::io", "I/O access outside sandbox", RiskLevel::Blocked),
    ("process::", "Process manipulation", RiskLevel::Blocked),
    ("std::os", "OS-level access", RiskLevel::Blocked),
    ("std::thread", "Thread spawning (denial-of-service risk)", RiskLevel::Suspicious),
    ("std::sync", "Synchronization primitive abuse", RiskLevel::Suspicious),
    ("std::env", "Environment variable access (secret leakage)", RiskLevel::Blocked),
    ("std::path", "Filesystem path manipulation", RiskLevel::Suspicious),
    ("std::ffi", "FFI calls (native code execution)", RiskLevel::Blocked),
    ("std::mem", "Memory manipulation", RiskLevel::Blocked),
    ("std::ptr", "Pointer manipulation", RiskLevel::Blocked),
    ("std::panic", "Panic/crash inducement", RiskLevel::Blocked),
    ("sys::", "System call access", RiskLevel::Blocked),
];

pub struct ScriptGuard;

impl ScriptGuard {
    pub fn inspect(script: &str, context: &InspectionContext) -> ScriptInspectionResult {
        let mut blocked = Vec::new();
        let mut max_risk = RiskLevel::Safe;

        for &(pattern, description, ref severity) in DANGEROUS_PATTERNS {
            if script.contains(pattern) {
                let risk = if *severity == RiskLevel::Blocked && !is_allowed_in_context(pattern, context) {
                    RiskLevel::Blocked
                } else {
                    (*severity).clone()
                };
                blocked.push(BlockedPattern {
                    pattern: pattern.to_string(),
                    description: description.to_string(),
                    severity: risk.clone(),
                });
                if risk == RiskLevel::Blocked {
                    max_risk = RiskLevel::Blocked;
                } else if max_risk != RiskLevel::Blocked && risk == RiskLevel::Suspicious {
                    max_risk = RiskLevel::Suspicious;
                }
            }
        }

        let allowed = max_risk != RiskLevel::Blocked;
        ScriptInspectionResult {
            allowed,
            blocked_patterns: blocked,
            risk_level: max_risk,
            script_id: context.script_id.clone(),
            inspector: "script_guard_v1".to_string(),
        }
    }
}

fn is_allowed_in_context(pattern: &str, _context: &InspectionContext) -> bool {
    let _ = pattern;
    false
}

#[derive(Debug, Clone, Default)]
pub struct InspectionContext {
    pub script_id: Option<String>,
    pub bot_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub allowed_features: Vec<String>,
}

pub struct AuditLogger;

impl AuditLogger {
    pub fn log_blocked(result: &ScriptInspectionResult) {
        let entry = serde_json::json!({
            "event": "script_blocked",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "inspector": result.inspector,
            "risk_level": result.risk_level,
            "script_id": result.script_id,
            "blocked_count": result.blocked_patterns.len(),
            "blocked_patterns": result.blocked_patterns.iter().map(|p| serde_json::json!({
                "pattern": p.pattern,
                "description": p.description,
                "severity": p.severity,
            })).collect::<Vec<_>>(),
        });
        log::warn!("Script blocked by security guard: {}", entry);
    }

    pub fn log_llm_call(record: &LlmCallRecord) {
        let entry = serde_json::json!({
            "event": "llm_call",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "bot_id": record.bot_id,
            "session_id": record.session_id,
            "model": record.model,
            "prompt_tokens": record.prompt_tokens,
            "completion_tokens": record.completion_tokens,
            "total_tokens": record.total_tokens,
            "latency_ms": record.latency_ms,
            "provider": record.provider,
        });
        log::info!("LLM call recorded: {}", entry);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallRecord {
    pub id: Uuid,
    pub bot_id: Option<String>,
    pub session_id: Option<String>,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub latency_ms: u64,
    pub provider: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStats {
    pub total_llm_calls: u64,
    pub total_tokens: u64,
    pub blocked_scripts: u64,
    pub active_sessions: u64,
    pub alerts_active: u64,
    pub top_models: Vec<(String, u64)>,
    pub recent_incidents: Vec<BlockedPattern>,
}
