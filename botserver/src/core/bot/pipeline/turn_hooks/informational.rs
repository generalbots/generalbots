//! Heuristic for turns that ask about the outside world.

/// Heuristic: does the user turn ask for news / current / informational
/// facts about the outside world? (#1835)
pub fn is_informational_request(text: &str) -> bool {
    let lower = text.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }
    let code_intent = [
        "code",
        "script",
        "program",
        "python",
        "node",
        "shell",
        "vm_exec",
        "terminal",
        "execute",
        "run ",
        "arquivo",
        "file",
        "pdf",
        "pptx",
        "xlsx",
        "csv",
        "planilha",
        "relatorio",
        "report",
    ]
    .iter()
    .any(|k| lower.contains(k));
    if code_intent {
        return false;
    }
    let keywords = [
        "news",
        "noticia",
        "notícias",
        "noticias",
        "headline",
        "manchete",
        "latest",
        "recent",
        "recente",
        "today",
        "hoje",
        "current",
        "atual",
        "weather",
        "clima",
        "forecast",
        "previsão",
        "previsao",
        "price",
        "preço",
        "preco",
        "cotação",
        "cotacao",
        "stock",
        "ação",
        "acoes",
        "market",
        "mercado",
        "bitcoin",
        "crypto",
        "dollar",
        "dólar",
        "dolar",
        "euro",
        "who is",
        "who's",
        "what is",
        "o que é",
        "o que e",
        "quem é",
        "quem e",
        "when did",
        "quando",
        "where is",
        "onde",
        "how much",
        "quanto custa",
        "update",
        "atualização",
        "atualizacao",
        "information",
        "informação",
        "informacao",
        "tell me",
        "fale sobre",
        "explain",
        "explique",
        "notícia",
    ];
    keywords.iter().any(|k| lower.contains(k))
}
