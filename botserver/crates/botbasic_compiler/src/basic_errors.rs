use std::collections::HashSet;
use std::fmt;

// ============================================================================
// BASIC Compiler Error Codes
// ============================================================================
// Cada erro do compilador BASIC tem um código único (E-BAS-XXXX) que pode
// ser referenciado na documentação (botbook) e nas mensagens de log.
//
// Tabela de Códigos de Erro (mantenha atualizada no botbook):
//
// | Código       | Descrição                                           | Exemplo                                    |
// |--------------|-----------------------------------------------------|--------------------------------------------|
// | E-BAS-0001   | Diretivas mutuamente exclusivas no mesmo script     | SET SCHEDULE + ON UPDATE OF no mesmo .bas |
// | E-BAS-0002   |    — reservado                                      |                                            |
// | E-BAS-0003   |    — reservado                                      |                                            |
// | E-BAS-0004   |    — reservado                                      |                                            |
// | E-BAS-0005   |    — reservado                                      |                                            |
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicError {
    pub code: &'static str,
    pub message: String,
}

impl BasicError {
    pub const fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }
}

impl fmt::Display for BasicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

// ============================================================================
// CÓDIGO: E-BAS-0001 — Diretivas mutuamente exclusivas
// ============================================================================
// Um script BASIC pode ter, no máximo, UMA directiva de modo de execução.
// Os seguintes modos são mutuamente exclusivos:
//
// | Directiva              | Modo                      | Conflita com                          |
// |------------------------|---------------------------|---------------------------------------|
// | SET SCHEDULE           | Agendado (cron)           | ON UPDATE OF, WEBHOOK, ON EMAIL, ON CHANGE |
// | ON UPDATE OF           | Gatilho de tabela (DB)    | SET SCHEDULE, WEBHOOK, ON EMAIL, ON CHANGE |
// | WEBHOOK                | Webhook HTTP              | SET SCHEDULE, ON UPDATE OF, ON EMAIL, ON CHANGE |
// | ON EMAIL FROM          | Gatilho de email          | SET SCHEDULE, ON UPDATE OF, WEBHOOK, ON CHANGE |
// | ON CHANGE              | Gatilho de mudança        | SET SCHEDULE, ON UPDATE OF, WEBHOOK, ON EMAIL |
// | USE WEBSITE            | Scraping                  | Nenhum (coexiste com todos)           |

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Directive {
    SetSchedule,
    OnUpdateOf,
    Webhook,
    UseWebsite,
    OnEmail,
    OnChange,
}

const TRIGGER_MODES: &[Directive] = &[
    Directive::SetSchedule,
    Directive::OnUpdateOf,
    Directive::Webhook,
    Directive::OnEmail,
    Directive::OnChange,
];

impl Directive {
    pub fn name(&self) -> &'static str {
        match self {
            Directive::SetSchedule => "SET SCHEDULE",
            Directive::OnUpdateOf => "ON UPDATE OF",
            Directive::Webhook => "WEBHOOK",
            Directive::UseWebsite => "USE WEBSITE",
            Directive::OnEmail => "ON EMAIL FROM",
            Directive::OnChange => "ON CHANGE",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Directive::SetSchedule => "Agenda execução do script por cron",
            Directive::OnUpdateOf => "Registra gatilho de tabela no banco de dados",
            Directive::Webhook => "Registra webhook HTTP",
            Directive::UseWebsite => "Faz scraping de site para contexto",
            Directive::OnEmail => "Registra gatilho de email recebido",
            Directive::OnChange => "Registra gatilho de mudança em tabela",
        }
    }
}

/// Valida que um script não possui directivas mutuamente exclusivas.
pub fn validate_directives(directives: &HashSet<Directive>) -> Result<(), BasicError> {
    let trigger_modes: Vec<&Directive> = TRIGGER_MODES
        .iter()
        .filter(|d| directives.contains(d))
        .collect();

    if trigger_modes.len() > 1 {
        let names: Vec<&str> = trigger_modes.iter().map(|d| d.name()).collect();
        return Err(BasicError::new(
            "E-BAS-0001",
            format!(
                "Diretivas mutuamente exclusivas encontradas: {}. \
                 Um script BASIC pode ter apenas UM modo de gatilho \
                 (agendamento, gatilho de tabela, webhook, email ou mudança).",
                names.join(" e ")
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e_bas_0001_schedule_and_on_update() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::SetSchedule);
        dirs.insert(Directive::OnUpdateOf);
        let err = validate_directives(&dirs).unwrap_err();
        assert_eq!(err.code, "E-BAS-0001");
    }

    #[test]
    fn test_e_bas_0001_website_ok() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::SetSchedule);
        dirs.insert(Directive::UseWebsite);
        assert!(validate_directives(&dirs).is_ok());
    }

    #[test]
    fn test_e_bas_0001_single_trigger_ok() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::OnUpdateOf);
        assert!(validate_directives(&dirs).is_ok());
    }

    #[test]
    fn test_error_display() {
        let err = BasicError::new("E-BAS-9999", "Teste".to_string());
        let msg = format!("{}", err);
        assert_eq!(msg, "[E-BAS-9999] Teste");
    }
}
