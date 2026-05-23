//! Ponto eletrônico via WhatsApp (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Marcação de ponto via WhatsApp com geolocalização
//! - Bate-papo de ponto: "bater ponto", "registrar entrada/saída"
//! - Histórico de ponto por colaborador
//! - Relatórios de horas trabalhadas, atrasos, horas extras
//! - Aprovação de folha de ponto por gestores
//! - Integração com sistemas de folha de pagamento
//! - Notificações de lembrete para bater ponto
//!
//! ## Gap Analysis
//! - Não há implementação atual de ponto eletrônico no código existente
//! - O módulo `whatsapp` em botserver/src/whatsapp/ pode ser estendido
//! - Uma nova tabela `time_entries` no banco PostgreSQL será necessária
//! - O módulo `timeseries` em botserver/src/timeseries/ pode armazenar registros
