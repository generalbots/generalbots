use crate::types::{VibeContext, VibeUseCase};
use std::collections::HashMap;

pub struct VibePromptManager {
    templates: HashMap<VibeUseCase, UseCaseTemplate>,
}

struct UseCaseTemplate {
    system_prompt: String,
    tool_instructions: String,
    output_format: String,
}

impl VibePromptManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(
            VibeUseCase::SoftwareDevelopment,
            UseCaseTemplate {
                system_prompt: "Você é um agente de desenvolvimento de software especializado. \
                    Analise requisitos de software, gere código em conformidade com as melhores práticas, \
                    revise alterações buscando defeitos e vulnerabilidades, e proponha correções precisas. \
                    Sempre responda no formato JSON estruturado conforme as instruções de ferramenta."
                    .to_string(),
                tool_instructions: "Ferramentas disponíveis: classify_intent, compile_plan, \
                    execute_plan, deploy_app, read_file, write_file, run_tests. \
                    Chame ferramentas usando JSON com campos: tool_name, arguments."
                    .to_string(),
                output_format: r#"{"tool_calls": [{"tool_name": "...", "arguments": {...}}]}"#.to_string(),
            },
        );
        templates.insert(
            VibeUseCase::CustomerSupport,
            UseCaseTemplate {
                system_prompt: "Você é um agente de atendimento ao cliente profissional. \
                    Resolva tickets de suporte de forma autônoma, consulte dados do CRM, \
                    identifique contatos e oportunidades, e forneça respostas contextualizadas. \
                    Mantenha cortesia e precisão em todas as interações."
                    .to_string(),
                tool_instructions: "Ferramentas disponíveis: search_contacts, get_deals, \
                    create_ticket, update_ticket, send_email, search_knowledge_base. \
                    Chame ferramentas usando JSON com campos: tool_name, arguments."
                    .to_string(),
                output_format: r#"{"tool_calls": [{"tool_name": "...", "arguments": {...}}]}"#.to_string(),
            },
        );
        templates.insert(
            VibeUseCase::FinancialAnalysis,
            UseCaseTemplate {
                system_prompt: "Você é um agente de análise financeira quantitative. \
                    Agregue indicadores de sentimento de mercado em tempo real, gere relatórios \
                    financeiros marcados por categoria de risco, identifique tendências e anomalias, \
                    e apresente resultados com precisão decimal adequada."
                    .to_string(),
                tool_instructions: "Ferramentas disponíveis: fetch_market_data, \
                    analyze_sentiment, generate_report, detect_anomalies, \
                    calculate_risk_metrics, search_historical_data. \
                    Chame ferramentas usando JSON com campos: tool_name, arguments."
                    .to_string(),
                output_format: r#"{"tool_calls": [{"tool_name": "...", "arguments": {...}}]}"#.to_string(),
            },
        );

        Self { templates }
    }

    pub fn build_context(&self, use_case: VibeUseCase, user_message: &str, history: &[crate::types::ContextMessage]) -> VibeContext {
        let mut ctx = VibeContext::new(uuid::Uuid::nil(), use_case);
        if let Some(template) = self.templates.get(&use_case) {
            ctx.system_prompt = format!(
                "{}\n\n{}\n\nFormato de saída esperado:\n{}",
                template.system_prompt, template.tool_instructions, template.output_format
            );
        }
        ctx.conversation_history = history.to_vec();
        ctx.add_user_message(user_message.to_string());
        ctx
    }

    pub fn system_prompt_for(&self, use_case: VibeUseCase) -> String {
        self.templates
            .get(&use_case)
            .map(|t| format!("{}\n\n{}\n\nFormato de saída esperado:\n{}", t.system_prompt, t.tool_instructions, t.output_format))
            .unwrap_or_else(|| use_case.default_system_prompt().to_string())
    }

    pub fn compose_prompt(&self, context: &VibeContext, user_message: &str) -> String {
        let mut parts = Vec::new();
        parts.push(format!("System: {}", context.system_prompt));

        for msg in &context.conversation_history {
            parts.push(format!("{}: {}", msg.role, msg.content));
        }

        if !context.kb_references.is_empty() {
            parts.push(format!("Contexto de base de conhecimento: {}", context.kb_references.join(", ")));
        }

        parts.push(format!("User: {user_message}"));
        parts.join("\n\n")
    }
}

impl Default for VibePromptManager {
    fn default() -> Self {
        Self::new()
    }
}
