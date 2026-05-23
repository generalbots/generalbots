//! Painel administrativo do CRM (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Dashboard central com KPIs em tempo real
//! - Gráficos de atendimento: volume, tempo médio, satisfação
//! - Gestão de atendentes: performance, carga, métricas individuais
//! - Configuração de filas, SLAs, horários de atendimento
//! - Auditoria de ações dos atendentes
//! - Relatórios exportáveis (PDF, CSV, XLSX)
//! - Gestão de Knowledge Base do CRM
//! - Logs de integração e webhooks
//!
//! ## Gap Analysis
//! - O módulo `admin` em botserver/src/admin/ possui painel administrativo geral
//! - O módulo `dashboards` em botserver/src/dashboards/ gerencia dashboards
//! - É necessário estender para métricas específicas de CRM
//! - O frontend HTMX em botui/ui/ precisa de novas páginas de administração
