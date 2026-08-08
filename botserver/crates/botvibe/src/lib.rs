pub mod types;
pub mod prompt_manager;
pub mod tool_executor;
pub mod telemetry;
pub mod api;
pub mod knowledge_graph;
pub mod agent_loop;
pub mod projects;
pub mod projects_api;
pub mod vm_lifecycle;
mod vm_incus;
pub mod vms_api;
pub mod publish;
pub mod caddy;
pub mod domains;
pub mod domains_api;
pub mod domains_tool;
pub mod harness;
pub mod ops;
pub mod ops_api;
pub mod ops_tools;
pub mod backups;
pub mod rbac;
pub mod members_api;
pub mod metering;
pub mod metering_api;
pub mod metering_schema;

pub use types::{
    VibeContext, VibeLlmOps, VibeProgressEvent, VibeRun, VibeRunConfig, VibeRunState, VibeState,
    VibeToolCall, VibeUseCase,
};
pub use prompt_manager::VibePromptManager;
pub use tool_executor::{VibeToolExecutor, ToolRegistry, ToolDescriptor, ToolSchema};
pub use telemetry::VibeTelemetry;
pub use api::router;
pub use agent_loop::AgentLoop;
pub use projects::{Project, ProjectKind, ProjectRegistry};
pub use projects_api::projects_router;
pub use rbac::{ProjectMember, ProjectRbac, ProjectRole};
pub use members_api::members_router;
pub use metering::{MeterPlan, UsageSummary, VMetering, VMeteringRef};
pub use metering_api::metering_router;
