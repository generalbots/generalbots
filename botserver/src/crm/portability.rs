//! Portabilidade numérica e integração SIP (Issue #517).
//!
//! ## SOON
//! Funcionalidades planejadas:
//! - Portabilidade de números telefônicos entre operadoras
//! - Integração com SIP trunk para chamadas VoIP
//! - Roteamento de chamadas baseado em regras de negócio
//! - Gravação de chamadas e armazenamento no Drive
//! - URA (Unidade de Resposta Audível) com Bot de IA
//! - Transferência de chamadas entre atendentes
//!
//! ## Gap Analysis
//! - Não há implementação atual de SIP/telefonia no código existente
//! - O módulo `channels` em botserver/src/channels/ pode ser estendido
//! - A integração com Meet/LiveKit já existe em botserver/src/meet/
