//! Webhooks e integração via API (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Webhooks para eventos do CRM (novo contato, ticket criado, status alterado)
//! - API REST para integração externa
//! - Autenticação via API Key ou JWT
//! - Rate limiting por integração
//! - Logs de chamadas de webhook com retry automático
//! - Fila de eventos para processamento assíncrono
//! - Status da entrega com notificação de falha
//!
//! ## Gap Analysis
//! - O módulo `core/api/routes.rs` gerencia rotas da API
//! - O módulo `webhooks` em botserver/src/auto_task/ pode ser referência
//! - É necessário implementar sistema de eventos e fila para webhooks
//! - O módulo `api` em botserver/src/api/ contém handlers existentes
