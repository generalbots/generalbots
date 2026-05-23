//! Assistente de IA para atendentes (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Assistente inline que sugere respostas durante o atendimento
//! - Pesquisa semântica na base de conhecimento (Knowledge Base)
//! - Resumo automático de conversas longas
//! - Sugestão de próxima ação baseada no contexto
//! - Detecção de intenção e sentimento do cliente
//! - Geração de wrap-up pós-atendimento
//! - Respostas rápidas configuráveis por bot
//!
//! ## Gap Analysis
//! - O módulo `llm` em botserver/src/llm/ possui integração com modelos de linguagem
//! - O módulo `core/bot/tool_context.rs` gerencia contexto de ferramentas
//! - O módulo `core/bot/ws_handler.rs` processa mensagens WebSocket
//! - É necessário conectar IA ao fluxo de atendimento humano
