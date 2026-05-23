//! Kanban de atendentes (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Quadro Kanban para visualização de atendimentos em tempo real
//! - Colunas: Pendente, Em Andamento, Aguardando Cliente, Resolvido
//! - Arrastar e soltar tickets entre colunas
//! - Limite de WIP (Work In Progress) por atendente
//! - Notificações em tempo real via WebSocket
//! - Métricas de produtividade por atendente
//! - Filtros por tipo de atendimento, prioridade, canal
//!
//! ## Gap Analysis
//! - Não há implementação atual de Kanban no código existente
//! - O frontend HTMX pode ser estendido com biblioteca de drag-and-drop
//! - WebSocket existente em botserver/src/websocket.rs pode ser reutilizado
