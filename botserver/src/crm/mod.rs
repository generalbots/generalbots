//! Módulo CRM (Issue #517).
//!
//! Este módulo implementa a revisão da especificação de CRM, cobrindo:
//!
//! ## Áreas de funcionalidade
//!
//! | Área | Módulo | Status |
//! |------|--------|--------|
//! | Portabilidade e integração SIP | `portability` | Planned |
//! | Canal WhatsApp Business | `whatsapp` | Planned |
//! | Bot de atendimento com IA | `ai_assistant` | Planned |
//! | Fila de atendimento humano | `queue` | Planned |
//! | Kanban de atendentes | `kanban` | Planned |
//! | CRM integrado (Contatos, Leads, Oportunidades, Pipeline) | `crm_core` | Planned |
//! | Tickets de suporte | `tickets` | Planned |
//! | Ponto eletrônico via WhatsApp | `time_tracking` | Planned |
//! | Webhooks e integração via API | `webhooks` | Planned |
//! | Painel administrativo | `admin` | Planned |
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
