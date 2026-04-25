use crate::core::shared::models::Switcher;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

const STANDARD_SWITCHER_IDS: &[&str] = &[
    "tables", "infographic", "cards", "list", "comparison", "timeline", "markdown", "chart",
];

fn get_switcher_prompt_map() -> &'static [(&'static str, &'static str)] {
    &[
        ("tables", "REGRAS DE FORMATO: SEMPRE retorne suas respostas em formato de tabela HTML usando <table>, <thead>, <tbody>, <tr>, <th>, <td>. Cada dado deve ser uma célula. Use cabeçalhos claros na primeira linha. Se houver dados numéricos, alinhe à direita. Se houver texto, alinhe à esquerda. Use cores sutis em linhas alternadas (nth-child). NÃO use markdown tables, use HTML puro."),
        ("infographic", "REGRAS DE FORMATO: Crie representações visuais HTML usando SVG, progress bars, stat cards, e elementos gráficos. Use elementos como: <svg> para gráficos, <div style=\"width:X%;background:color\"> para barras de progresso, ícones emoji, badges coloridos. Organize informações visualmente com grids, flexbox, e espaçamento. Inclua legendas e rótulos visuais claros."),
        ("cards", "REGRAS DE FORMATO: Retorne informações em formato de cards HTML. Cada card deve ter: <div class=\"card\" style=\"border:1px solid #ddd;border-radius:8px;padding:16px;margin:8px;box-shadow:0 2px 4px rgba(0,0,0,0.1)\">. Dentro do card use: título em <h3> ou <strong>, subtítulo em <p> style=\"color:#666\", ícone emoji ou ícone SVG no topo, badges de status. Organize cards em grid usando display:grid ou flex-wrap."),
        ("list", "REGRAS DE FORMATO: Use apenas listas HTML: <ul> para bullets e <ol> para números numerados. Cada item em <li>. Use sublistas aninhadas quando apropriado. NÃO use parágrafos de texto, converta tudo em itens de lista. Adicione ícones emoji no início de cada <li> quando possível. Use classes CSS para estilização: .list-item, .sub-list."),
        ("comparison", "REGRAS DE FORMATO: Crie comparações lado a lado em HTML. Use grid de 2 colunas: <div style=\"display:grid;grid-template-columns:1fr 1fr;gap:20px\">. Cada lado em uma <div class=\"comparison-side\"> com borda colorida distinta. Use headers claros para cada lado. Adicione seção de \"Diferenças Chave\" com bullet points. Use cores contrastantes para cada lado (ex: azul vs laranja). Inclua tabela de comparação resumida no final."),
        ("timeline", "REGRAS DE FORMATO: Organize eventos cronologicamente em formato de timeline HTML. Use <div class=\"timeline\"> com border-left vertical. Cada evento em <div class=\"timeline-item\"> com: data em <span class=\"timeline-date\" style=\"font-weight:bold;color:#666\">, título em <h3>, descrição em <p>. Adicione círculo indicador na timeline line. Ordene do mais antigo para o mais recente. Use espaçamento claro entre eventos."),
        ("markdown", "REGRAS DE FORMATO: Use exclusivamente formato Markdown padrão. Sintaxe permitida: **negrito**, *itálico*, `inline code`, ```bloco de código```, # cabeçalhos, - bullets, 1. números, [links](url), ![alt](url), | tabela | markdown |. NÃO use HTML tags exceto para blocos de código. Siga estritamente a sintaxe CommonMark."),
        ("chart", "REGRAS DE FORMATO: Crie gráficos e diagramas em HTML SVG. Use elementos SVG: <svg width=\"X\" height=\"Y\">, <line> para gráficos de linha, <rect> para gráficos de barra, <circle> para gráficos de pizza, <path> para gráficos de área. Inclua eixos com labels, grid lines, legendas. Use cores distintas para cada série de dados (ex: vermelho, azul, verde). Adicione tooltips com valores ao hover."),
    ]
}

pub fn resolve_switcher_prompt(switcher_id: &str) -> Option<String> {
    for (id, prompt) in get_switcher_prompt_map() {
        if *id == switcher_id {
            return Some((*prompt).to_string());
        }
    }
    None
}

fn is_standard_switcher(id: &str) -> bool {
    STANDARD_SWITCHER_IDS.contains(&id)
}

fn get_redis_connection(cache_client: &Arc<redis::Client>) -> Option<redis::Connection> {
    let timeout = Duration::from_millis(50);
    cache_client.get_connection_with_timeout(timeout).ok()
}

pub fn clear_switchers_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();

    engine
        .register_custom_syntax(["CLEAR", "SWITCHERS"], true, move |_context, _inputs| {
            if let Some(cache_client) = &cache {
                let redis_key = format!("switchers:{}:{}", user_session.bot_id, user_session.id);
                let mut conn = match get_redis_connection(cache_client) {
                    Some(conn) => conn,
                    None => {
                        trace!("Cache not ready, skipping clear switchers");
                        return Ok(Dynamic::UNIT);
                    }
                };

                let result: Result<i64, redis::RedisError> =
                    redis::cmd("DEL").arg(&redis_key).query(&mut conn);

                match result {
                    Ok(deleted) => {
                        trace!(
                            "Cleared {} switchers from session {}",
                            deleted,
                            user_session.id
                        );
                    }
                    Err(e) => error!("Failed to clear switchers from Redis: {}", e),
                }
            } else {
                trace!("No cache configured, switchers not cleared");
            }

            Ok(Dynamic::UNIT)
        })
        .expect("valid syntax registration");
}

pub fn add_switcher_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let cache = state.cache.clone();
    let user_session_clone = user_session.clone();

    engine
        .register_custom_syntax(
            ["ADD_SWITCHER", "$expr$", "as", "$expr$"],
            true,
            move |context, inputs| {
                let switcher_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let button_text = context.eval_expression_tree(&inputs[1])?.to_string();

                add_switcher(
                    cache.as_ref(),
                    &user_session_clone,
                    &switcher_id,
                    &button_text,
                )?;

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");
}

fn add_switcher(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    first_param: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    let (switcher_id, switcher_prompt) = if is_standard_switcher(first_param) {
        (first_param.to_string(), resolve_switcher_prompt(first_param))
    } else {
        let custom_id = format!("custom:{}", simple_hash(first_param));
        (custom_id, Some(first_param.to_string()))
    };

    trace!(
        "ADD_SWITCHER: id={}, label={}, is_standard={}",
        switcher_id,
        button_text,
        is_standard_switcher(first_param)
    );

    if let Some(cache_client) = cache {
        let redis_key = format!("switchers:{}:{}", user_session.bot_id, user_session.id);

        let switcher_data = json!({
            "id": switcher_id,
            "label": button_text,
            "prompt": switcher_prompt,
            "is_standard": is_standard_switcher(first_param),
            "original_param": first_param
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add switcher");
                return Ok(());
            }
        };

        let _: Result<i64, redis::RedisError> = redis::cmd("SADD")
            .arg(&redis_key)
            .arg(switcher_data.to_string())
            .query(&mut conn);

        trace!(
            "Added switcher '{}' ({}) to session {}",
            switcher_id,
            if is_standard_switcher(first_param) { "standard" } else { "custom" },
            user_session.id
        );
    } else {
        trace!("No cache configured, switcher not added");
    }

    Ok(())
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

pub fn get_switchers(
    cache: Option<&Arc<redis::Client>>,
    bot_id: &str,
    session_id: &str,
) -> Vec<Switcher> {
    let mut switchers = Vec::new();

    if let Some(cache_client) = cache {
        let redis_key = format!("switchers:{}:{}", bot_id, session_id);

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, returning empty switchers");
                return switchers;
            }
        };

        let result: Result<Vec<String>, redis::RedisError> =
            redis::cmd("SMEMBERS").arg(&redis_key).query(&mut conn);

        match result {
            Ok(items) => {
                for item in items {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&item) {
                        let switcher = Switcher::new(
                            json["id"].as_str().unwrap_or(""),
                            json["label"].as_str().unwrap_or(""),
                        )
                        .with_prompt(json["prompt"].as_str().unwrap_or(""));
                        switchers.push(switcher);
                    }
                }
                info!(
                    "Retrieved {} switchers for session {}",
                    switchers.len(),
                    session_id
                );
            }
            Err(e) => error!("Failed to get switchers from Redis: {}", e),
        }
    }

    switchers
}

pub fn resolve_active_switchers(
    cache: Option<&Arc<redis::Client>>,
    bot_id: &str,
    session_id: &str,
    active_ids: &[String],
) -> String {
    if active_ids.is_empty() {
        return String::new();
    }

    let stored_switchers = get_switchers(cache, bot_id, session_id);
    let mut prompts: Vec<String> = Vec::new();

    for id in active_ids {
        let prompt = stored_switchers
            .iter()
            .find(|s| s.id == *id)
            .and_then(|s| s.prompt.clone())
            .or_else(|| resolve_switcher_prompt(id));

        if let Some(p) = prompt {
            if !p.is_empty() {
                prompts.push(p);
            }
        }
    }

    prompts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_standard_switcher() {
        assert!(is_standard_switcher("tables"));
        assert!(is_standard_switcher("chart"));
        assert!(!is_standard_switcher("my_custom"));
    }

    #[test]
    fn test_resolve_standard_prompt() {
        let prompt = resolve_switcher_prompt("tables");
        assert!(prompt.is_some());
        assert!(prompt.unwrap().contains("tabela HTML"));
    }

    #[test]
    fn test_resolve_unknown_returns_none() {
        let prompt = resolve_switcher_prompt("nonexistent");
        assert!(prompt.is_none());
    }

    #[test]
    fn test_custom_switcher_id() {
        let id = if is_standard_switcher("use quadrados") {
            "use quadrados".to_string()
        } else {
            format!("custom:{}", simple_hash("use quadrados"))
        };
        assert!(id.starts_with("custom:"));
    }
}
