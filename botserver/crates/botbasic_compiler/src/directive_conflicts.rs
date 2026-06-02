use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Directive {
    SetSchedule,
    OnUpdateOf,
    Webhook,
    UseWebsite,
    OnEmail,
    OnChange,
}

pub const ALL_DIRECTIVES: &[Directive] = &[
    Directive::SetSchedule,
    Directive::OnUpdateOf,
    Directive::Webhook,
    Directive::UseWebsite,
    Directive::OnEmail,
    Directive::OnChange,
];

/// Máquina de modos de execução únicos.
/// Um script BASIC pode ter no máximo UM destes modos de gatilho:
///
/// | Directiva              | Modo                      | Mutuamente exclusivo com              |
/// |------------------------|---------------------------|---------------------------------------|
/// | SET SCHEDULE           | Agendado (cron)           | ON UPDATE OF, WEBHOOK, ON EMAIL, ON CHANGE |
/// | ON UPDATE OF           | Gatilho de tabela (DB)    | SET SCHEDULE, WEBHOOK, ON EMAIL, ON CHANGE |
/// | WEBHOOK                | Webhook HTTP              | SET SCHEDULE, ON UPDATE OF, ON EMAIL, ON CHANGE |
/// | ON EMAIL               | Gatilho de email          | SET SCHEDULE, ON UPDATE OF, WEBHOOK, ON CHANGE |
/// | ON CHANGE              | Gatilho de mudança        | SET SCHEDULE, ON UPDATE OF, WEBHOOK, ON EMAIL |
/// | USE WEBSITE            | Scraping de site          | Nenhum (pode coexistir com todos)     |
const TRIGGER_MODES: &[Directive] = &[
    Directive::SetSchedule,
    Directive::OnUpdateOf,
    Directive::Webhook,
    Directive::OnEmail,
    Directive::OnChange,
];

/// Retorna `true` se duas directivas são mutuamente exclusivas.
pub fn are_conflicting(a: Directive, b: Directive) -> bool {
    if a == b {
        return false;
    }
    // USE_WEBSITE não conflita com nada
    if a == Directive::UseWebsite || b == Directive::UseWebsite {
        return false;
    }
    // Qualquer par de modos de gatilho conflita
    true
}

/// Valida um conjunto de directivas encontradas num script.
/// Retorna `Ok(())` se não houver conflitos, ou `Err` com descrição do conflito.
pub fn validate_directives(directives: &HashSet<Directive>) -> Result<(), String> {
    let trigger_modes: Vec<&Directive> = TRIGGER_MODES
        .iter()
        .filter(|d| directives.contains(d))
        .collect();

    if trigger_modes.len() > 1 {
        let names: Vec<&str> = trigger_modes.iter().map(|d| d.name()).collect();
        return Err(format!(
            "Diretivas mutuamente exclusivas encontradas: {}. \
             Um script BASIC pode ter apenas UM modo de gatilho \
             (agendamento, gatilho de tabela, webhook, email ou mudança).",
            names.join(" e ")
        ));
    }

    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_conflict_with_website() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::SetSchedule);
        dirs.insert(Directive::UseWebsite);
        assert!(validate_directives(&dirs).is_ok());
    }

    #[test]
    fn test_conflict_schedule_and_on_update() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::SetSchedule);
        dirs.insert(Directive::OnUpdateOf);
        assert!(validate_directives(&dirs).is_err());
    }

    #[test]
    fn test_single_trigger_is_ok() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::OnUpdateOf);
        assert!(validate_directives(&dirs).is_ok());
    }

    #[test]
    fn test_website_with_any_is_ok() {
        let mut dirs = HashSet::new();
        dirs.insert(Directive::UseWebsite);
        dirs.insert(Directive::Webhook);
        assert!(validate_directives(&dirs).is_ok());

        let mut dirs2 = HashSet::new();
        dirs2.insert(Directive::UseWebsite);
        dirs2.insert(Directive::OnUpdateOf);
        assert!(validate_directives(&dirs2).is_ok());
    }
}
