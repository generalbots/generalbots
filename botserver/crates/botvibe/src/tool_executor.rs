use crate::types::{VibeToolCall, VibeToolResult, VibeUseCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub requires_approval: bool,
    pub allowed_use_cases: Vec<VibeUseCase>,
}

impl ToolSchema {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
            requires_approval: false,
            allowed_use_cases: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_parameters(mut self, params: serde_json::Value) -> Self {
        self.parameters = params;
        self
    }

    #[must_use]
    pub fn with_approval(mut self) -> Self {
        self.requires_approval = true;
        self
    }

    #[must_use]
    pub fn with_use_cases(mut self, cases: Vec<VibeUseCase>) -> Self {
        self.allowed_use_cases = cases;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub schema: ToolSchema,
    pub category: ToolCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Autotask,
    Deployment,
    Crm,
    Sources,
    File,
    Analysis,
}

pub type ToolHandler = Arc<dyn Fn(serde_json::Value) -> ToolFuture + Send + Sync>;
type ToolFuture = std::pin::Pin<Box<dyn std::future::Future<Output = VibeToolResult> + Send>>;

pub struct ToolRegistry {
    tools: RwLock<HashMap<String, RegisteredTool>>,
}

struct RegisteredTool {
    descriptor: ToolDescriptor,
    handler: ToolHandler,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let registry = Self {
            tools: RwLock::new(HashMap::new()),
        };
        registry.register_builtin_tools();
        registry
    }

    fn register_builtin_tools(&self) {
        let mut tools = futures::executor::block_on(self.tools.write());

        let autotask_tools = vec![
            ("classify_intent", "Classifica a intenção do usuário usando o motor AutoTask", false),
            ("compile_plan", "Compila um plano de execução a partir da intenção classificada", false),
            ("execute_plan", "Executa um plano compilado de forma supervisionada", true),
            ("create_and_execute", "Classifica, compila e executa em um único fluxo", true),
        ];

        for (name, desc, approval) in autotask_tools {
            let schema = ToolSchema::new(name, desc)
                .with_approval_if(approval)
                .with_use_cases(vec![VibeUseCase::SoftwareDevelopment, VibeUseCase::CustomerSupport]);
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Autotask },
                handler: Arc::new(stub_tool_handler(name)),
            });
        }

        let deploy_tools = vec![
            ("deploy_app", "Realiza deploy de aplicação gerada", true),
        ];

        for (name, desc, approval) in deploy_tools {
            let schema = ToolSchema::new(name, desc)
                .with_approval_if(approval)
                .with_use_cases(vec![VibeUseCase::SoftwareDevelopment]);
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Deployment },
                handler: Arc::new(stub_tool_handler(name)),
            });
        }

        let crm_tools = vec![
            ("search_contacts", "Busca contatos no CRM", false),
            ("get_deals", "Obtém oportunidades do pipeline CRM", false),
            ("create_ticket", "Cria ticket de suporte", false),
            ("update_ticket", "Atualiza status de ticket", false),
            ("send_email", "Envia e-mail ao contato", false),
        ];

        for (name, desc, approval) in crm_tools {
            let schema = ToolSchema::new(name, desc)
                .with_approval_if(approval)
                .with_use_cases(vec![VibeUseCase::CustomerSupport]);
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Crm },
                handler: Arc::new(stub_tool_handler(name)),
            });
        }

        let analysis_tools = vec![
            ("fetch_market_data", "Obtém dados de mercado em tempo real", false),
            ("analyze_sentiment", "Analisa sentimento de mercado", false),
            ("generate_report", "Gera relatório financeiro marcado", false),
            ("detect_anomalies", "Detecta anomalias em séries temporais", false),
        ];

        for (name, desc, approval) in analysis_tools {
            let schema = ToolSchema::new(name, desc)
                .with_approval_if(approval)
                .with_use_cases(vec![VibeUseCase::FinancialAnalysis]);
            tools.insert(name.to_string(), RegisteredTool {
                descriptor: ToolDescriptor { schema, category: ToolCategory::Analysis },
                handler: Arc::new(stub_tool_handler(name)),
            });
        }
    }

    pub async fn register(&self, descriptor: ToolDescriptor, handler: ToolHandler) {
        let name = descriptor.schema.name.clone();
        let mut tools = self.tools.write().await;
        tools.insert(name, RegisteredTool { descriptor, handler });
    }

    pub async fn get_descriptor(&self, name: &str) -> Option<ToolDescriptor> {
        let tools = self.tools.read().await;
        tools.get(name).map(|t| t.descriptor.clone())
    }

    pub async fn list_tools(&self) -> Vec<ToolDescriptor> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.descriptor.clone()).collect()
    }

    pub async fn list_tools_for_use_case(&self, use_case: VibeUseCase) -> Vec<ToolDescriptor> {
        let tools = self.tools.read().await;
        tools.values()
            .filter(|t| {
                t.descriptor.schema.allowed_use_cases.is_empty()
                    || t.descriptor.schema.allowed_use_cases.contains(&use_case)
            })
            .map(|t| t.descriptor.clone())
            .collect()
    }

    pub async fn validate_arguments(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<(), String> {
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name).ok_or_else(|| format!("Ferramenta '{tool_name}' não encontrada"))?;

        if tool.descriptor.schema.parameters.get("properties").is_none() {
            return Ok(());
        }

        if let Some(props) = tool.descriptor.schema.parameters.get("properties").and_then(|p| p.as_object()) {
            let empty_map = serde_json::Map::new();
            let args_map = arguments.as_object().unwrap_or(&empty_map);
            if let Some(required) = tool.descriptor.schema.parameters.get("required").and_then(|r| r.as_array()) {
                for req in required {
                    let key = req.as_str().unwrap_or("");
                    if !args_map.contains_key(key) {
                        return Err(format!("Parâmetro obrigatório ausente: '{key}'"));
                    }
                }
            }
            for key in args_map.keys() {
                if !props.contains_key(key) {
                    return Err(format!("Parâmetro desconhecido: '{key}'"));
                }
            }
        }

        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

trait ToolSchemaExt {
    fn with_approval_if(self, needs_approval: bool) -> Self;
}

impl ToolSchemaExt for ToolSchema {
    fn with_approval_if(mut self, needs_approval: bool) -> Self {
        self.requires_approval = needs_approval;
        self
    }
}

fn stub_tool_handler(name: &'static str) -> impl Fn(serde_json::Value) -> ToolFuture + Send + Sync + 'static {
    move |_args: serde_json::Value| {
        let name = name.to_string();
        Box::pin(async move {
            VibeToolResult {
                success: true,
                data: serde_json::json!({"stub": true, "tool": name}),
                error: None,
                latency_ms: 0,
            }
        })
    }
}

pub struct VibeToolExecutor {
    registry: Arc<ToolRegistry>,
}

impl VibeToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        tool_call: &mut VibeToolCall,
        use_case: VibeUseCase,
    ) -> Result<(), String> {
        let descriptor = self.registry.get_descriptor(&tool_call.tool_name).await
            .ok_or_else(|| format!("Ferramenta '{}' não registrada", tool_call.tool_name))?;

        if !descriptor.schema.allowed_use_cases.is_empty()
            && !descriptor.schema.allowed_use_cases.contains(&use_case) {
            return Err(format!("Ferramenta '{}' não disponível para caso de uso {}", tool_call.tool_name, use_case));
        }

        self.registry.validate_arguments(&tool_call.tool_name, &tool_call.arguments).await?;

        if descriptor.schema.requires_approval && !tool_call.approved {
            tool_call.requires_approval = true;
            return Err("Aprovação requerida antes da execução".to_string());
        }

        let start = std::time::Instant::now();
        let tools = self.registry.tools.read().await;
        let result = if let Some(registered) = tools.get(&tool_call.tool_name) {
            let handler = registered.handler.clone();
            drop(tools);
            (handler)(tool_call.arguments.clone()).await
        } else {
            drop(tools);
            VibeToolResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(format!("Ferramenta '{}' não encontrada", tool_call.tool_name)),
                latency_ms: 0,
            }
        };

        let latency = start.elapsed().as_millis() as u64;
        tool_call.result = Some(VibeToolResult {
            latency_ms: latency,
            ..result
        });

        Ok(())
    }

    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }
}
