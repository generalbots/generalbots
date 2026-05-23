//! Módulo CRM (Issue #517).
//!
//! Implementa o núcleo do sistema de CRM:
//!
//! | Área | Módulo | Status |
//! |------|--------|--------|
//! | Portabilidade e integração SIP | `portability` | Implementado |
//! | Canal WhatsApp Business | `whatsapp` | Implementado |
//! | Bot de atendimento com IA | `ai_assistant` | Implementado |
//! | Fila de atendimento humano | `queue` | Implementado |
//! | Kanban de atendentes | `kanban` | Implementado |
//! | CRM integrado (Contatos, Leads, Oportunidades, Pipeline) | `crm_core` | Implementado |
//! | Tickets de suporte | `tickets` | Implementado |
//! | Ponto eletrônico via WhatsApp | `time_tracking` | Implementado |
//! | Webhooks e integração via API | `webhooks` | Implementado |
//! | Painel administrativo | `admin` | Implementado |
//!
//! ## Stack
//! - Backend: Rust/Axum
//! - Banco: PostgreSQL (via Diesel)
//! - Armazenamento: MinIO (Drive)
//! - Cache: Valkey (Redis)
//! - Cache vetorial: Qdrant
//! - Frontend: HTMX + WebSocket

pub mod portability;
pub mod whatsapp;
pub mod queue;
pub mod kanban;
pub mod ai_assistant;
pub mod crm_core;
pub mod tickets;
pub mod time_tracking;
pub mod webhooks;
pub mod admin;

pub use crm_core::{Contact, Lead, LeadStatus, Opportunity, Pipeline};
pub use queue::AttendanceQueue;
pub use tickets::{Ticket, TicketPriority, TicketStatus};
pub use admin::DashboardMetrics;
