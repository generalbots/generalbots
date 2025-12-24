use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttendanceCommand {
    CheckIn,
    CheckOut,
    Break,
    Resume,
    Status,
    Report,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordConfig {
    pub enabled: bool,
    pub case_sensitive: bool,
    pub prefix: Option<String>,
    pub keywords: HashMap<String, AttendanceCommand>,
    pub aliases: HashMap<String, String>,
}

impl Default for KeywordConfig {
    fn default() -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("checkin".to_string(), AttendanceCommand::CheckIn);
        keywords.insert("checkout".to_string(), AttendanceCommand::CheckOut);
        keywords.insert("break".to_string(), AttendanceCommand::Break);
        keywords.insert("resume".to_string(), AttendanceCommand::Resume);
        keywords.insert("status".to_string(), AttendanceCommand::Status);
        keywords.insert("report".to_string(), AttendanceCommand::Report);
        keywords.insert("override".to_string(), AttendanceCommand::Override);

        let mut aliases = HashMap::new();
        aliases.insert("in".to_string(), "checkin".to_string());
        aliases.insert("out".to_string(), "checkout".to_string());
        aliases.insert("pause".to_string(), "break".to_string());
        aliases.insert("continue".to_string(), "resume".to_string());
        aliases.insert("stat".to_string(), "status".to_string());

        Self {
            enabled: true,
            case_sensitive: false,
            prefix: Some("!".to_string()),
            keywords,
            aliases,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub command: AttendanceCommand,
    pub args: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub raw_input: String,
}

#[derive(Debug, Clone)]
pub struct KeywordParser {
    config: Arc<RwLock<KeywordConfig>>,
}

impl KeywordParser {
    pub fn new(config: KeywordConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn parse(&self, input: &str) -> Option<ParsedCommand> {
        let config = self.config.read().await;

        if !config.enabled {
            return None;
        }

        let processed_input = if config.case_sensitive {
            input.trim().to_string()
        } else {
            input.trim().to_lowercase()
        };

        let command_text = if let Some(prefix) = &config.prefix {
            if !processed_input.starts_with(prefix) {
                return None;
            }
            processed_input.strip_prefix(prefix)?
        } else {
            &processed_input
        };

        let parts: Vec<&str> = command_text.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command_word = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        let resolved_command = if let Some(alias) = config.aliases.get(command_word) {
            alias.as_str()
        } else {
            command_word
        };

        let command = config.keywords.get(resolved_command)?;

        Some(ParsedCommand {
            command: command.clone(),
            args,
            timestamp: Utc::now(),
            raw_input: input.to_string(),
        })
    }

    pub async fn update_config(&self, config: KeywordConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    pub async fn add_keyword(&self, keyword: String, command: AttendanceCommand) {
        let mut config = self.config.write().await;
        config.keywords.insert(keyword, command);
    }

    pub async fn add_alias(&self, alias: String, target: String) {
        let mut config = self.config.write().await;
        config.aliases.insert(alias, target);
    }

    pub async fn remove_keyword(&self, keyword: &str) -> bool {
        let mut config = self.config.write().await;
        config.keywords.remove(keyword).is_some()
    }

    pub async fn remove_alias(&self, alias: &str) -> bool {
        let mut config = self.config.write().await;
        config.aliases.remove(alias).is_some()
    }

    pub async fn get_config(&self) -> KeywordConfig {
        self.config.read().await.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub id: String,
    pub user_id: String,
    pub command: AttendanceCommand,
    pub timestamp: DateTime<Utc>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttendanceService {
    parser: Arc<KeywordParser>,
    records: Arc<RwLock<Vec<AttendanceRecord>>>,
}

impl AttendanceService {
    pub fn new(parser: KeywordParser) -> Self {
        Self {
            parser: Arc::new(parser),
            records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn process_input(&self, user_id: &str, input: &str) -> Result<AttendanceResponse> {
        let parsed = self
            .parser
            .parse(input)
            .await
            .ok_or_else(|| anyhow!("No valid command found in input"))?;

        match parsed.command {
            AttendanceCommand::CheckIn => self.handle_check_in(user_id, &parsed).await,
            AttendanceCommand::CheckOut => self.handle_check_out(user_id, &parsed).await,
            AttendanceCommand::Break => self.handle_break(user_id, &parsed).await,
            AttendanceCommand::Resume => self.handle_resume(user_id, &parsed).await,
            AttendanceCommand::Status => self.handle_status(user_id).await,
            AttendanceCommand::Report => self.handle_report(user_id, &parsed).await,
            AttendanceCommand::Override => self.handle_override(user_id, &parsed).await,
        }
    }

    async fn handle_check_in(
        &self,
        user_id: &str,
        parsed: &ParsedCommand,
    ) -> Result<AttendanceResponse> {
        let mut records = self.records.write().await;

        if let Some(last_record) = records.iter().rev().find(|r| r.user_id == user_id) {
            if matches!(last_record.command, AttendanceCommand::CheckIn) {
                return Ok(AttendanceResponse::Error {
                    message: "Already checked in".to_string(),
                });
            }
        }

        let record = AttendanceRecord {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            command: AttendanceCommand::CheckIn,
            timestamp: parsed.timestamp,
            location: parsed.args.first().cloned(),
            notes: if parsed.args.len() > 1 {
                Some(parsed.args[1..].join(" "))
            } else {
                None
            },
        };

        let time = Local::now().format("%H:%M").to_string();
        records.push(record);

        Ok(AttendanceResponse::Success {
            message: format!("Checked in at {}", time),
            timestamp: parsed.timestamp,
        })
    }

    async fn handle_check_out(
        &self,
        user_id: &str,
        parsed: &ParsedCommand,
    ) -> Result<AttendanceResponse> {
        let mut records = self.records.write().await;

        let check_in_time = records
            .iter()
            .rev()
            .find(|r| r.user_id == user_id && matches!(r.command, AttendanceCommand::CheckIn))
            .map(|r| r.timestamp);

        if check_in_time.is_none() {
            return Ok(AttendanceResponse::Error {
                message: "Not checked in".to_string(),
            });
        }

        let record = AttendanceRecord {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            command: AttendanceCommand::CheckOut,
            timestamp: parsed.timestamp,
            location: parsed.args.first().cloned(),
            notes: if parsed.args.len() > 1 {
                Some(parsed.args[1..].join(" "))
            } else {
                None
            },
        };

        let duration = parsed.timestamp - check_in_time.unwrap();
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;

        records.push(record);

        Ok(AttendanceResponse::Success {
            message: format!("Checked out. Total time: {}h {}m", hours, minutes),
            timestamp: parsed.timestamp,
        })
    }

    async fn handle_break(
        &self,
        user_id: &str,
        parsed: &ParsedCommand,
    ) -> Result<AttendanceResponse> {
        let mut records = self.records.write().await;

        let is_checked_in = records
            .iter()
            .rev()
            .find(|r| r.user_id == user_id)
            .map(|r| matches!(r.command, AttendanceCommand::CheckIn))
            .unwrap_or(false);

        if !is_checked_in {
            return Ok(AttendanceResponse::Error {
                message: "Not checked in".to_string(),
            });
        }

        let record = AttendanceRecord {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            command: AttendanceCommand::Break,
            timestamp: parsed.timestamp,
            location: None,
            notes: parsed.args.first().cloned(),
        };

        let time = Local::now().format("%H:%M").to_string();
        records.push(record);

        Ok(AttendanceResponse::Success {
            message: format!("Break started at {}", time),
            timestamp: parsed.timestamp,
        })
    }

    async fn handle_resume(
        &self,
        user_id: &str,
        parsed: &ParsedCommand,
    ) -> Result<AttendanceResponse> {
        let mut records = self.records.write().await;

        let break_time = records
            .iter()
            .rev()
            .find(|r| r.user_id == user_id && matches!(r.command, AttendanceCommand::Break))
            .map(|r| r.timestamp);

        if break_time.is_none() {
            return Ok(AttendanceResponse::Error {
                message: "Not on break".to_string(),
            });
        }

        let record = AttendanceRecord {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            command: AttendanceCommand::Resume,
            timestamp: parsed.timestamp,
            location: None,
            notes: None,
        };

        let duration = parsed.timestamp - break_time.unwrap();
        let minutes = duration.num_minutes();

        records.push(record);

        Ok(AttendanceResponse::Success {
            message: format!("Resumed work. Break duration: {} minutes", minutes),
            timestamp: parsed.timestamp,
        })
    }

    async fn handle_status(&self, user_id: &str) -> Result<AttendanceResponse> {
        let records = self.records.read().await;

        let user_records: Vec<_> = records.iter().filter(|r| r.user_id == user_id).collect();

        if user_records.is_empty() {
            return Ok(AttendanceResponse::Status {
                status: "No records found".to_string(),
                details: None,
            });
        }

        let last_record = user_records.last().unwrap();
        let status = match last_record.command {
            AttendanceCommand::CheckIn => "Checked in",
            AttendanceCommand::CheckOut => "Checked out",
            AttendanceCommand::Break => "On break",
            AttendanceCommand::Resume => "Working",
            _ => "Unknown",
        };

        let details = format!(
            "Last action: {} at {}",
            status,
            last_record.timestamp.format("%Y-%m-%d %H:%M:%S")
        );

        Ok(AttendanceResponse::Status {
            status: status.to_string(),
            details: Some(details),
        })
    }

    async fn handle_report(
        &self,
        user_id: &str,
        _parsed: &ParsedCommand,
    ) -> Result<AttendanceResponse> {
        let records = self.records.read().await;

        let user_records: Vec<_> = records.iter().filter(|r| r.user_id == user_id).collect();

        if user_records.is_empty() {
            return Ok(AttendanceResponse::Report {
                data: "No attendance records found".to_string(),
            });
        }

        let mut report = String::new();
        report.push_str(&format!("Attendance Report for User: {}\n", user_id));
        report.push_str("========================\n");

        for record in user_records {
            let action = match record.command {
                AttendanceCommand::CheckIn => "Check In",
                AttendanceCommand::CheckOut => "Check Out",
                AttendanceCommand::Break => "Break",
                AttendanceCommand::Resume => "Resume",
                _ => "Other",
            };

            report.push_str(&format!(
                "{}: {} at {}\n",
                record.timestamp.format("%Y-%m-%d %H:%M:%S"),
                action,
                record.location.as_deref().unwrap_or("N/A")
            ));
        }

        Ok(AttendanceResponse::Report { data: report })
    }

    async fn handle_override(
        &self,
        user_id: &str,
        parsed: &ParsedCommand,
    ) -> Result<AttendanceResponse> {
        if parsed.args.len() < 2 {
            return Ok(AttendanceResponse::Error {
                message: "Override requires target user and action".to_string(),
            });
        }

        let target_user = &parsed.args[0];
        let action = &parsed.args[1];

        log::warn!(
            "Override command by {} for user {}: {}",
            user_id,
            target_user,
            action
        );

        Ok(AttendanceResponse::Success {
            message: format!("Override applied for user {}", target_user),
            timestamp: parsed.timestamp,
        })
    }

    pub async fn get_user_records(&self, user_id: &str) -> Vec<AttendanceRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect()
    }

    pub async fn clear_records(&self) {
        let mut records = self.records.write().await;
        records.clear();
    }

    pub async fn get_today_work_time(&self, user_id: &str) -> Duration {
        let records = self.records.read().await;
        let today = Local::now().date_naive();

        let mut total_duration = Duration::zero();
        let mut last_checkin: Option<DateTime<Utc>> = None;

        for record in records.iter().filter(|r| r.user_id == user_id) {
            if record.timestamp.with_timezone(&Local).date_naive() != today {
                continue;
            }

            match record.command {
                AttendanceCommand::CheckIn => {
                    last_checkin = Some(record.timestamp);
                }
                AttendanceCommand::CheckOut => {
                    if let Some(checkin) = last_checkin {
                        total_duration = total_duration + (record.timestamp - checkin);
                        last_checkin = None;
                    }
                }
                _ => {}
            }
        }

        if let Some(checkin) = last_checkin {
            total_duration = total_duration + (Utc::now() - checkin);
        }

        total_duration
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttendanceResponse {
    Success {
        message: String,
        timestamp: DateTime<Utc>,
    },
    Error {
        message: String,
    },
    Status {
        status: String,
        details: Option<String>,
    },
    Report {
        data: String,
    },
}
