use std::collections::HashMap;
use std::sync::Arc;

use botvibe::tool_executor::{
    ToolCategory, ToolDescriptor, ToolHandler, ToolRegistry, ToolSchema, ToolSchemaExt,
    VibeToolExecutor,
};
use botvibe::types::{
    DbPool, VibeProgressEvent, VibeRun, VibeRunSignal, VibeState, VibeToolCall, VibeToolResult,
    VibeUseCase,
};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

struct MockState {
    runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
}

impl MockState {
    fn new() -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl VibeState for MockState {
    fn db_pool(&self) -> &DbPool {
        unreachable!("db_pool not exercised in tool executor tests")
    }
    fn broadcast_progress(&self, _event: VibeProgressEvent) {}
    fn progress_sender(&self) -> Option<&broadcast::Sender<VibeProgressEvent>> {
        None
    }
    fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, VibeRun>>> {
        &self.runs
    }
    fn run_signal_sender(&self) -> Option<&broadcast::Sender<VibeRunSignal>> {
        None
    }
}

#[test]
fn tool_schema_new_and_builders() {
    let schema = ToolSchema::new("echo", "Echo input");
    assert_eq!(schema.name, "echo");
    assert_eq!(schema.description, "Echo input");
    assert!(!schema.requires_approval);
    assert!(schema.allowed_use_cases.is_empty());
    assert_eq!(schema.parameters["type"], "object");

    let schema = ToolSchema::new("echo", "Echo")
        .with_parameters(serde_json::json!({"type": "object", "properties": {"x": {"type": "string"}}}))
        .with_approval()
        .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]);
    assert!(schema.requires_approval);
    assert_eq!(
        schema.allowed_use_cases,
        vec![VibeUseCase::SoftwareDevelopment]
    );
    assert_eq!(schema.parameters["properties"]["x"]["type"], "string");

    let schema = ToolSchema::new("echo", "Echo").with_approval_if(true);
    assert!(schema.requires_approval);
    let schema = ToolSchema::new("echo", "Echo").with_approval_if(false);
    assert!(!schema.requires_approval);
}

#[tokio::test]
async fn registry_registers_builtin_and_harness_tools() {
    let registry = ToolRegistry::new();
    let tools = registry.list_tools().await;
    assert!(!tools.is_empty());

    let names: Vec<String> = tools.iter().map(|t| t.schema.name.clone()).collect();
    for expected in [
        "classify_intent",
        "compile_plan",
        "execute_plan",
        "create_and_execute",
        "deploy_app",
        "publish/project",
        "domain/bind",
        "domain/verify",
        "domain/tls",
        "file/read",
        "file/write",
        "shell/run",
        "git/status",
        "logs/read",
        "test/run",
        "search_contacts",
        "fetch_market_data",
    ] {
        assert!(names.contains(&expected.to_string()), "missing tool {expected}");
    }

    let file_tool = tools.iter().find(|t| t.schema.name == "file/read").unwrap();
    assert_eq!(file_tool.category, ToolCategory::File);
    assert!(!file_tool.schema.requires_approval);

    let write_tool = tools.iter().find(|t| t.schema.name == "file/write").unwrap();
    assert!(write_tool.schema.requires_approval);

    let deploy_tool = tools.iter().find(|t| t.schema.name == "deploy_app").unwrap();
    assert_eq!(deploy_tool.category, ToolCategory::Deployment);
    assert!(deploy_tool.schema.requires_approval);
}

#[tokio::test]
async fn registry_lists_tools_for_use_case() {
    let registry = ToolRegistry::new();
    let dev_tools = registry.list_tools_for_use_case(VibeUseCase::SoftwareDevelopment).await;
    let dev_names: Vec<&str> = dev_tools.iter().map(|t| t.schema.name.as_str()).collect();
    assert!(dev_names.contains(&"file/read"));
    assert!(dev_names.contains(&"deploy_app"));
    assert!(!dev_names.contains(&"fetch_market_data"));

    let finance_tools = registry.list_tools_for_use_case(VibeUseCase::FinancialAnalysis).await;
    let finance_names: Vec<&str> = finance_tools.iter().map(|t| t.schema.name.as_str()).collect();
    assert!(finance_names.contains(&"fetch_market_data"));
    assert!(finance_names.contains(&"analyze_sentiment"));
    assert!(!finance_names.contains(&"file/read"));
    assert!(!finance_names.contains(&"deploy_app"));
}

#[tokio::test]
async fn validate_arguments_enforces_required_and_unknown() {
    let registry = ToolRegistry::new();
    let ok = registry
        .validate_arguments(
            "deploy_app",
            &serde_json::json!({"app_name": "demo", "org": "gen", "project_type": "bot"}),
        )
        .await;
    assert!(ok.is_ok());

    let missing = registry
        .validate_arguments("deploy_app", &serde_json::json!({"app_name": "demo"}))
        .await;
    assert!(missing.is_err());
    assert!(missing.unwrap_err().contains("org"));

    let unknown = registry
        .validate_arguments(
            "deploy_app",
            &serde_json::json!({"app_name": "a", "org": "o", "project_type": "bot", "bogus": 1}),
        )
        .await;
    assert!(unknown.is_err());
    assert!(unknown.unwrap_err().contains("bogus"));

    let unknown_tool = registry.validate_arguments("nope", &serde_json::json!({})).await;
    assert!(unknown_tool.is_err());
}

#[tokio::test]
async fn executor_rejects_unregistered_tool() {
    let registry = Arc::new(ToolRegistry::new());
    let executor = VibeToolExecutor::new(registry);
    let mut call = VibeToolCall::new(
        Uuid::new_v4(),
        "no/such/tool".into(),
        serde_json::json!({}),
        false,
    );
    let err = executor
        .execute(&mut call, VibeUseCase::SoftwareDevelopment, &MockState::new())
        .await;
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("não registrada"));
}

#[tokio::test]
async fn executor_blocks_tool_for_wrong_use_case() {
    let registry = Arc::new(ToolRegistry::new());
    let executor = VibeToolExecutor::new(registry);
    let mut call = VibeToolCall::new(
        Uuid::new_v4(),
        "deploy_app".into(),
        serde_json::json!({"app_name": "a", "org": "o", "project_type": "bot"}),
        false,
    );
    let err = executor
        .execute(&mut call, VibeUseCase::FinancialAnalysis, &MockState::new())
        .await;
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("não disponível"));
}

#[tokio::test]
async fn executor_requires_approval_before_execution() {
    let registry = Arc::new(ToolRegistry::new());
    let executor = VibeToolExecutor::new(registry);
    let mut call = VibeToolCall::new(
        Uuid::new_v4(),
        "file/write".into(),
        serde_json::json!({"project": "demo", "path": "a.txt", "content": "x"}),
        false,
    );
    let err = executor
        .execute(&mut call, VibeUseCase::SoftwareDevelopment, &MockState::new())
        .await;
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("Aprovação"));
    assert!(call.requires_approval);
}

#[tokio::test]
async fn executor_marks_missing_required_parameter() {
    let registry = Arc::new(ToolRegistry::new());
    let executor = VibeToolExecutor::new(registry);
    let mut call = VibeToolCall::new(
        Uuid::new_v4(),
        "file/read".into(),
        serde_json::json!({"path": "a.txt"}),
        false,
    );
    let err = executor
        .execute(&mut call, VibeUseCase::SoftwareDevelopment, &MockState::new())
        .await;
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("project"));
}

#[tokio::test]
async fn executor_records_result_after_successful_custom_tool() {
    let registry = Arc::new(ToolRegistry::new());
    let handler: ToolHandler = Arc::new(|_args, _state| {
        Box::pin(async {
            VibeToolResult {
                success: true,
                data: serde_json::json!({"done": true}),
                error: None,
                latency_ms: 0,
            }
        })
    });
    registry
        .register(
            ToolDescriptor {
                schema: ToolSchema::new("custom/probe", "Probe tool")
                    .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]),
                category: ToolCategory::Analysis,
            },
            handler,
        )
        .await;

    let executor = VibeToolExecutor::new(registry);
    let mut call = VibeToolCall::new(
        Uuid::new_v4(),
        "custom/probe".into(),
        serde_json::json!({}),
        false,
    );
    let result = executor
        .execute(&mut call, VibeUseCase::SoftwareDevelopment, &MockState::new())
        .await;
    assert!(result.is_ok());
    let out = call.result.expect("result recorded after execution");
    assert!(out.success);
    assert_eq!(out.data["done"], true);
}

#[test]
fn tool_descriptor_serializes_category_snake_case() {
    let descriptor = ToolDescriptor {
        schema: ToolSchema::new("t", "desc"),
        category: ToolCategory::Deployment,
    };
    let value = serde_json::to_value(&descriptor).unwrap();
    assert_eq!(value["category"], "deployment");
    assert_eq!(value["schema"]["name"], "t");
}
