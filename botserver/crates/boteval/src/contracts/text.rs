/// Contract matching is about meaning, not typography. Code identifiers use
/// underscores (`binary_search`), inflected languages change the ending
/// (`físico`/`física`, `aprender`/`aprendem`) and numbers are written either
/// as digits or words (`6`/`six`). A byte compare fails all of those even
/// when the answer is correct.
fn normalize_words(text: &str) -> String {
    text.to_lowercase()
        .replace(['_', '-', '—'], " ")
        // Split on every non-alphanumeric character, not just whitespace:
        // `**Bug:**`, `search(arr,` and `target):` must tokenize to the words
        // the contract names.
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn number_value(token: &str) -> Option<u32> {
    const WORDS: [&str; 21] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
        "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
        "sixteen", "seventeen", "eighteen", "nineteen", "twenty",
    ];
    const PT_WORDS: [&str; 21] = [
        "zero", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito",
        "nove", "dez", "onze", "doze", "treze", "catorze", "quinze", "dezesseis",
        "dezessete", "dezoito", "dezenove", "vinte",
    ];
    if let Ok(value) = token.parse::<u32>() {
        return Some(value);
    }
    WORDS
        .iter()
        .position(|word| *word == token)
        .or_else(|| PT_WORDS.iter().position(|word| *word == token))
        .map(|index| index as u32)
}

fn stem(token: &str) -> String {
    let mut word = token.to_string();
    // Longest suffixes first so "reutilizáveis" and "complexidade" reduce to
    // the same root as their verb/adjective forms.
    for suffix in [
        "mente", "ções", "áveis", "aveis", "idade", "dade", "mentos", "mento", "ados",
        "adas", "ado", "ada", "idos", "idas", "ido", "ida", "ções", "ção", "ões",
        "ão", "ais", "eis", "veis", "ável", "avel", "ible", "able", "icos", "icas",
        "ico", "ica", "ing", "ed", "es", "em", "ar", "er", "ir", "al", "s",
    ] {
        if word.len() > suffix.len() + 3 && word.ends_with(suffix) {
            word.truncate(word.len() - suffix.len());
            break;
        }
    }
    // Gender/number endings that survive the loop (`física`, `pausa`).
    while word.len() > 4 && matches!(word.chars().last(), Some('a' | 'o' | 'e')) {
        word.pop();
    }
    word
}

/// Common conversational equivalences between English and Portuguese, and
/// between formal/informal register. These are meanings, not spellings:
/// a correct answer that says "I'm doing well" satisfies "glad".
fn synonyms(token: &str) -> &'static [&'static str] {
    match token {
        "glad" | "happy" | "alegria" | "feliz" => {
            &["glad", "happy", "well", "good", "fine", "alegria", "feliz", "bem"]
        }
        "help" | "assist" | "support" | "ajudar" | "ajuda" => {
            &["help", "assist", "support", "ajudar", "ajuda", "auxiliar"]
        }
        "unable" | "cannot" | "cant" => &["unable", "cannot", "cant", "impossivel", "impossível"],
        "questions" | "perguntas" | "duvidas" | "dúvidas" => {
            &["questions", "perguntas", "duvidas", "dúvidas"]
        }
        "break" | "pause" | "rest" | "pausa" | "pausas" => {
            &["break", "pause", "rest", "pausa", "pausas"]
        }
        "symbols" | "characters" | "simbolos" | "símbolos" | "símbolo" => {
            &["symbols", "characters", "simbolos", "símbolos", "símbolo"]
        }
        "privacy" | "privacidade" => &["privacy", "privacidade"],
        "description" | "descricao" | "descrição" => {
            &["description", "descricao", "descrição", "details", "detalhes"]
        }
        "replace" | "substituir" => &["replace", "substituir", "substitua"],
        "assets" | "ativos" => &["assets", "ativos"],
        "liabilities" | "passivos" => &["liabilities", "passivos"],
        "progress" | "progresso" | "andamento" => &["progress", "progresso", "andamento", "status"],
        "blockers" | "impedimentos" | "bloqueios" => {
            &["blockers", "impedimentos", "bloqueios", "obstaculos", "obstáculos"]
        }
        "test" | "teste" | "testes" => &["test", "teste", "testes", "tested", "testing"],
        "reuse" | "reutilizar" | "reutilizaveis" | "reutilizáveis" => {
            &["reuse", "reutilizar", "reutilizaveis", "reutilizáveis", "reusable"]
        }
        "recycle" | "reciclar" => &["recycle", "reciclar", "reciclagem"],
        "thank" | "thanks" | "obrigado" | "agradeco" | "agradeço" => {
            &["thank", "thanks", "obrigado", "agradeco", "agradeço", "agradecido"]
        }
        // A bug report can name the remedy as a noun or a verb: "a correção"
        // and "como corrigir" are the same answer.
        "fix" | "fixes" | "correct" | "corrigir" | "correcao" | "correção" => {
            &[
                "fix",
                "fixes",
                "correct",
                "corrigir",
                "corrige",
                "correcao",
                "correção",
                "remedy",
            ]
        }
        _ => &[],
    }
}

fn token_matches(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    if let (Some(a), Some(b)) = (number_value(left), number_value(right)) {
        return a == b;
    }
    // A one-letter variable in a code contract (`n * 2`) names the same
    // operation as any other one-letter variable (`x * 2`).
    if left.len() == 1 && right.len() == 1 {
        return left.chars().all(|c| c.is_alphabetic())
            && right.chars().all(|c| c.is_alphabetic());
    }
    let (a, b) = (stem(left), stem(right));
    if a == b && a.len() >= 3 {
        return true;
    }
    // One-hop synonym lookup over BOTH sides. Checking only the left side
    // missed cases where the response used the synonym (`doing well` for
    // `glad`) instead of the contract word.
    synonyms(left)
        .iter()
        .chain(synonyms(right).iter())
        .any(|candidate| {
            let candidate = stem(candidate);
            candidate == a || candidate == b
        })
}

pub(super) fn contains_phrase(response: &str, phrase: &str) -> bool {
    let phrase_tokens: Vec<String> = normalize_words(phrase)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if phrase_tokens.is_empty() {
        return true;
    }
    let response_tokens: Vec<String> = normalize_words(response)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if phrase_tokens.len() == 1 {
        return response_tokens
            .iter()
            .any(|token| token_matches(token, &phrase_tokens[0]));
    }
    response_tokens
        .windows(phrase_tokens.len())
        .any(|window| {
            window
                .iter()
                .zip(phrase_tokens.iter())
                .all(|(left, right)| token_matches(left, right))
        })
}

pub(super) fn detect_language(text: &str) -> String {
    let text = text.to_lowercase();
    // Word/phrase markers rather than single letters: short answers such as
    // "A hexagon has 6 sides." or "Um hexágono tem 6 lados." must classify
    // correctly, and a numeric answer must stay "und" (inconclusive).
    let pt_markers = [
        " olá", " tudo", "ção", "ões", "ão", " não", " você", " é ", " que", " para",
        " uma", " com", " exemplo", " como", " dos", " das", " por", " mais", " quando",
        " porque", " isso", " esta", " este", " tem ", " lado", " resídu", " fórmula",
        " corresponde", " quilômetro", " milha", " depende", " sistema", " cor", " vermelho",
        " verde", " azul", " substitu", " plástico", " reciclar", " coleta", " seletiva",
        " adeus", " obrigado", " tchau", " português", " hexágono", " dobrada", " lista",
        " em ", " não ",
    ];
    let en_markers = [
        " hello", " how", " the ", " and ", " is ", " are ", " you", " your", " for ",
        " with ", " this ", " that ", " has ", " have ", " sides", " days", " energy",
        " wind", " solar", " hydropower", " goodbye", " thank", " what", " which", " does",
        " year", " leap", " hexagon", " miles", " degrees", " temperature", " good",
    ];
    let pt = pt_markers.iter().filter(|m| text.contains(*m)).count();
    let en = en_markers.iter().filter(|m| text.contains(*m)).count();
    if pt > en {
        "pt-BR".into()
    } else if en > pt {
        "en".into()
    } else {
        "und".into()
    }
}
