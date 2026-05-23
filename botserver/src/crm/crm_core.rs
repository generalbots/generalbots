//! CRM integrado: Contatos, Leads, Oportunidades e Pipeline (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Gestão de contatos com enriquecimento automático
//! - Pipeline de vendas com estágios configuráveis
//! - Leads com pontuação automática (score)
//! - Oportunidades vinculadas a contatos e pipeline
//! - Histórico de interações por contato
//! - Importação/exportação via CSV e XLSX
//! - Integração com email (Stalwart) e calendário
//! - Relatórios e dashboards de vendas
//!
//! ## Gap Analysis
//! - O módulo `contacts` em botserver/src/contacts/ possui modelo de contatos
//! - O módulo `people` em botserver/src/people/ gerencia pessoas físicas
//! - O módulo `core/shared/models/core.rs` define modelos Diesel
//! - É necessário implementar pipeline, leads e oportunidades
//! - A integração com email (Stalwart) está em botserver/src/email/
