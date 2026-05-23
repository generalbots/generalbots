//! Tickets de suporte (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Criação de tickets via múltiplos canais (WhatsApp, Web, Email)
//! - Classificação automática por IA (categoria, prioridade, atendente)
//! - Ciclo de vida do ticket: Novo, Atribuído, Em Andamento, Resolvido, Fechado
//! - SLAs com notificações de violação
//! - Respostas automáticas baseadas em Knowledge Base
//! - Satisfação do cliente (CSAT/NPS) pós-resolução
//! - Relatórios de tickets por período, categoria, atendente
//!
//! ## Gap Analysis
//! - O módulo `tickets` em botserver/src/tickets/ possui implementação inicial
//! - O módulo `attendance` em botserver/src/attendance/ gerencia atendimentos
//! - É necessário integrar com classificação por IA e SLAs
