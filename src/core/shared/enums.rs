//! Database Enum Types for Billion-Scale Schema
//!
//! This module defines Rust enums that map directly to PostgreSQL enum types.
//! Using enums instead of TEXT columns provides:
//! - Type safety at compile time
//! - Efficient storage (stored as integers internally)
//! - Fast comparisons and indexing
//! - Automatic validation
//!
//! All enums derive necessary traits for Diesel ORM integration.

use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
use std::io::Write;

// ============================================================================
// CHANNEL TYPES
// ============================================================================

/// Communication channel types for bot interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum ChannelType {
    Web = 0,
    WhatsApp = 1,
    Telegram = 2,
    MsTeams = 3,
    Slack = 4,
    Email = 5,
    Sms = 6,
    Voice = 7,
    Instagram = 8,
    Api = 9,
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::Web
    }
}

impl ToSql<SmallInt, Pg> for ChannelType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for ChannelType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Web),
            1 => Ok(Self::WhatsApp),
            2 => Ok(Self::Telegram),
            3 => Ok(Self::MsTeams),
            4 => Ok(Self::Slack),
            5 => Ok(Self::Email),
            6 => Ok(Self::Sms),
            7 => Ok(Self::Voice),
            8 => Ok(Self::Instagram),
            9 => Ok(Self::Api),
            _ => Err(format!("Unknown ChannelType: {}", value).into()),
        }
    }
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web => write!(f, "web"),
            Self::WhatsApp => write!(f, "whatsapp"),
            Self::Telegram => write!(f, "telegram"),
            Self::MsTeams => write!(f, "msteams"),
            Self::Slack => write!(f, "slack"),
            Self::Email => write!(f, "email"),
            Self::Sms => write!(f, "sms"),
            Self::Voice => write!(f, "voice"),
            Self::Instagram => write!(f, "instagram"),
            Self::Api => write!(f, "api"),
        }
    }
}

impl std::str::FromStr for ChannelType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(Self::Web),
            "whatsapp" => Ok(Self::WhatsApp),
            "telegram" => Ok(Self::Telegram),
            "msteams" | "ms_teams" | "teams" => Ok(Self::MsTeams),
            "slack" => Ok(Self::Slack),
            "email" => Ok(Self::Email),
            "sms" => Ok(Self::Sms),
            "voice" => Ok(Self::Voice),
            "instagram" => Ok(Self::Instagram),
            "api" => Ok(Self::Api),
            _ => Err(format!("Unknown channel type: {}", s)),
        }
    }
}

// ============================================================================
// MESSAGE ROLE
// ============================================================================

/// Role of a message in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum MessageRole {
    User = 1,
    Assistant = 2,
    System = 3,
    Tool = 4,
    Episodic = 9,
    Compact = 10,
}

impl Default for MessageRole {
    fn default() -> Self {
        Self::User
    }
}

impl ToSql<SmallInt, Pg> for MessageRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for MessageRole {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            1 => Ok(Self::User),
            2 => Ok(Self::Assistant),
            3 => Ok(Self::System),
            4 => Ok(Self::Tool),
            9 => Ok(Self::Episodic),
            10 => Ok(Self::Compact),
            _ => Err(format!("Unknown MessageRole: {}", value).into()),
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Assistant => write!(f, "assistant"),
            Self::System => write!(f, "system"),
            Self::Tool => write!(f, "tool"),
            Self::Episodic => write!(f, "episodic"),
            Self::Compact => write!(f, "compact"),
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            "tool" => Ok(Self::Tool),
            "episodic" => Ok(Self::Episodic),
            "compact" => Ok(Self::Compact),
            _ => Err(format!("Unknown message role: {}", s)),
        }
    }
}

// ============================================================================
// MESSAGE TYPE
// ============================================================================

/// Type of message content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum MessageType {
    Text = 0,
    Image = 1,
    Audio = 2,
    Video = 3,
    Document = 4,
    Location = 5,
    Contact = 6,
    Sticker = 7,
    Reaction = 8,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Text
    }
}

impl ToSql<SmallInt, Pg> for MessageType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for MessageType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Text),
            1 => Ok(Self::Image),
            2 => Ok(Self::Audio),
            3 => Ok(Self::Video),
            4 => Ok(Self::Document),
            5 => Ok(Self::Location),
            6 => Ok(Self::Contact),
            7 => Ok(Self::Sticker),
            8 => Ok(Self::Reaction),
            _ => Err(format!("Unknown MessageType: {}", value).into()),
        }
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
            Self::Audio => write!(f, "audio"),
            Self::Video => write!(f, "video"),
            Self::Document => write!(f, "document"),
            Self::Location => write!(f, "location"),
            Self::Contact => write!(f, "contact"),
            Self::Sticker => write!(f, "sticker"),
            Self::Reaction => write!(f, "reaction"),
        }
    }
}

// ============================================================================
// LLM PROVIDER
// ============================================================================

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum LlmProvider {
    OpenAi = 0,
    Anthropic = 1,
    AzureOpenAi = 2,
    AzureClaude = 3,
    Google = 4,
    Local = 5,
    Ollama = 6,
    Groq = 7,
    Mistral = 8,
    Cohere = 9,
}

impl Default for LlmProvider {
    fn default() -> Self {
        Self::OpenAi
    }
}

impl ToSql<SmallInt, Pg> for LlmProvider {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for LlmProvider {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::OpenAi),
            1 => Ok(Self::Anthropic),
            2 => Ok(Self::AzureOpenAi),
            3 => Ok(Self::AzureClaude),
            4 => Ok(Self::Google),
            5 => Ok(Self::Local),
            6 => Ok(Self::Ollama),
            7 => Ok(Self::Groq),
            8 => Ok(Self::Mistral),
            9 => Ok(Self::Cohere),
            _ => Err(format!("Unknown LlmProvider: {}", value).into()),
        }
    }
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::AzureOpenAi => write!(f, "azure_openai"),
            Self::AzureClaude => write!(f, "azure_claude"),
            Self::Google => write!(f, "google"),
            Self::Local => write!(f, "local"),
            Self::Ollama => write!(f, "ollama"),
            Self::Groq => write!(f, "groq"),
            Self::Mistral => write!(f, "mistral"),
            Self::Cohere => write!(f, "cohere"),
        }
    }
}

// ============================================================================
// CONTEXT PROVIDER (Vector DB)
// ============================================================================

/// Supported vector database providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum ContextProvider {
    None = 0,
    Qdrant = 1,
    Pinecone = 2,
    Weaviate = 3,
    Milvus = 4,
    PgVector = 5,
    Elasticsearch = 6,
}

impl Default for ContextProvider {
    fn default() -> Self {
        Self::Qdrant
    }
}

impl ToSql<SmallInt, Pg> for ContextProvider {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for ContextProvider {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Qdrant),
            2 => Ok(Self::Pinecone),
            3 => Ok(Self::Weaviate),
            4 => Ok(Self::Milvus),
            5 => Ok(Self::PgVector),
            6 => Ok(Self::Elasticsearch),
            _ => Err(format!("Unknown ContextProvider: {}", value).into()),
        }
    }
}

// ============================================================================
// TASK STATUS
// ============================================================================

/// Status of a task (both regular tasks and auto-tasks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum TaskStatus {
    Pending = 0,
    Ready = 1,
    Running = 2,
    Paused = 3,
    WaitingApproval = 4,
    Completed = 5,
    Failed = 6,
    Cancelled = 7,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl ToSql<SmallInt, Pg> for TaskStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for TaskStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Ready),
            2 => Ok(Self::Running),
            3 => Ok(Self::Paused),
            4 => Ok(Self::WaitingApproval),
            5 => Ok(Self::Completed),
            6 => Ok(Self::Failed),
            7 => Ok(Self::Cancelled),
            _ => Err(format!("Unknown TaskStatus: {}", value).into()),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Ready => write!(f, "ready"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::WaitingApproval => write!(f, "waiting_approval"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "ready" => Ok(Self::Ready),
            "running" => Ok(Self::Running),
            "paused" => Ok(Self::Paused),
            "waiting_approval" | "waitingapproval" => Ok(Self::WaitingApproval),
            "completed" | "done" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown task status: {}", s)),
        }
    }
}

// ============================================================================
// TASK PRIORITY
// ============================================================================

/// Priority level for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
    Critical = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl ToSql<SmallInt, Pg> for TaskPriority {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for TaskPriority {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Low),
            1 => Ok(Self::Normal),
            2 => Ok(Self::High),
            3 => Ok(Self::Urgent),
            4 => Ok(Self::Critical),
            _ => Err(format!("Unknown TaskPriority: {}", value).into()),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "normal" | "medium" => Ok(Self::Normal),
            "high" => Ok(Self::High),
            "urgent" => Ok(Self::Urgent),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("Unknown task priority: {}", s)),
        }
    }
}

// ============================================================================
// EXECUTION MODE
// ============================================================================

/// Execution mode for autonomous tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum ExecutionMode {
    Manual = 0,
    Supervised = 1,
    Autonomous = 2,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Supervised
    }
}

impl ToSql<SmallInt, Pg> for ExecutionMode {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for ExecutionMode {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Manual),
            1 => Ok(Self::Supervised),
            2 => Ok(Self::Autonomous),
            _ => Err(format!("Unknown ExecutionMode: {}", value).into()),
        }
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Supervised => write!(f, "supervised"),
            Self::Autonomous => write!(f, "autonomous"),
        }
    }
}

// ============================================================================
// RISK LEVEL
// ============================================================================

/// Risk assessment level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum RiskLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::Low
    }
}

impl ToSql<SmallInt, Pg> for RiskLevel {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for RiskLevel {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Low),
            2 => Ok(Self::Medium),
            3 => Ok(Self::High),
            4 => Ok(Self::Critical),
            _ => Err(format!("Unknown RiskLevel: {}", value).into()),
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

// ============================================================================
// APPROVAL STATUS
// ============================================================================

/// Status of an approval request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum ApprovalStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Expired = 3,
    Skipped = 4,
}

impl Default for ApprovalStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl ToSql<SmallInt, Pg> for ApprovalStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for ApprovalStatus {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Approved),
            2 => Ok(Self::Rejected),
            3 => Ok(Self::Expired),
            4 => Ok(Self::Skipped),
            _ => Err(format!("Unknown ApprovalStatus: {}", value).into()),
        }
    }
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Rejected => write!(f, "rejected"),
            Self::Expired => write!(f, "expired"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

// ============================================================================
// APPROVAL DECISION
// ============================================================================

/// Decision made on an approval request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "snake_case")]
#[repr(i16)]
pub enum ApprovalDecision {
    Approve = 0,
    Reject = 1,
    Skip = 2,
}

impl ToSql<SmallInt, Pg> for ApprovalDecision {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for ApprovalDecision {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Approve),
            1 => Ok(Self::Reject),
            2 => Ok(Self::Skip),
            _ => Err(format!("Unknown ApprovalDecision: {}", value).into()),
        }
    }
}

impl std::fmt::Display for ApprovalDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Approve => write!(f, "approve"),
            Self::Reject => write!(f, "reject"),
            Self::Skip => write!(f, "skip"),
        }
    }
}

// ============================================================================
// INTENT TYPE
// ============================================================================

/// Classified intent type from user requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i16)]
pub enum IntentType {
    Unknown = 0,
    AppCreate = 1,
    Todo = 2,
    Monitor = 3,
    Action = 4,
    Schedule = 5,
    Goal = 6,
    Tool = 7,
    Query = 8,
}

impl Default for IntentType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl ToSql<SmallInt, Pg> for IntentType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        out.write_all(&v.to_be_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<SmallInt, Pg> for IntentType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::AppCreate),
            2 => Ok(Self::Todo),
            3 => Ok(Self::Monitor),
            4 => Ok(Self::Action),
            5 => Ok(Self::Schedule),
            6 => Ok(Self::Goal),
            7 => Ok(Self::Tool),
            8 => Ok(Self::Query),
            _ => Err(format!("Unknown IntentType: {}", value).into()),
        }
    }
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::AppCreate => write!(f, "APP_CREATE"),
            Self::Todo => write!(f, "TODO"),
            Self::Monitor => write!(f, "MONITOR"),
            Self::Action => write!(f, "ACTION"),
            Self::Schedule => write!(f, "SCHEDULE"),
            Self::Goal => write!(f, "GOAL"),
            Self::Tool => write!(f, "TOOL"),
            Self::Query => write!(f, "QUERY"),
        }
    }
}

impl std::str::FromStr for IntentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "UNKNOWN" => Ok(Self::Unknown),
            "APP_CREATE" | "APPCREATE" | "APP" | "APPLICATION" | "CREATE_APP" => Ok(Self::AppCreate),
            "TODO" | "TASK" | "REMINDER" => Ok(Self::Todo),
            "MONITOR" | "WATCH" | "ALERT" | "ON_CHANGE" => Ok(Self::Monitor),
