use serde_json::{json, Value};

/// Turns an explicit "open/read <url>" request into a browser proposal. This
/// is the producer the desktop build lacked: the assistant can act on a URL
/// without the model having to know an HTTP endpoint.
pub fn propose_from_text(text: &str) -> Option<Value> {
    let lower = text.to_lowercase();
    let intent = [
        "open ",
        "abrir ",
        "read ",
        "leia ",
        "extract ",
        "extraia ",
        "visit ",
        "aceda ",
        "navigate to ",
        "acesse ",
    ]
    .iter()
    .any(|verb| lower.contains(verb));
    if !intent {
        return None;
    }
    let url = text
        .split_whitespace()
        .find(|word| word.starts_with("http://") || word.starts_with("https://"))?
        .trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ')' | ']' | '>'))
        .to_string();
    Some(json!({
        "action": "open_url",
        "url": url,
        "extract": "title_and_text",
    }))
}

/// Turns an explicit "run/execute <command>" request into a terminal
/// proposal (#1645). A backticked command wins; otherwise the text after the
/// verb is used. The consent card still gates execution.
pub fn propose_terminal_from_text(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    let lower = trimmed.to_lowercase();
    // A turn that names the live-terminal tool belongs to the model: the
    // native `terminal_run` streams output into the run card and lets the
    // user take over. Proposing a consent card here skipped the model and
    // turned the phrasing itself into a shell command ("Run this with
    // terminal_run …" executed `this with terminal_run …`).
    if lower.contains("terminal_run") {
        return None;
    }
    let prefixes = [
        "run the command ",
        "execute the command ",
        "use vm_exec to run ",
        "use vm_exec to execute ",
        "use the terminal to run ",
        "run ",
        "execute ",
        "rode ",
        "roda ",
        "executar ",
        "executa ",
    ];
    let rest = prefixes.iter().find_map(|prefix| {
        lower.starts_with(prefix).then(|| {
            trimmed
                .chars()
                .skip(prefix.chars().count())
                .collect::<String>()
        })
    })?;
    let rest = rest.trim();
    let command = if let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        after.find('`').map(|end| after[..end].trim().to_string())?
    } else {
        // "Run this shell command with vm_exec: ls /nope. Report the exit
        // code" used to propose the whole sentence as the command. When the
        // text names a command/vm_exec before a colon, the payload after the
        // colon is the command; follow-up instructions are cut off.
        let candidate = match rest.split_once(':') {
            Some((head, after)) => {
                let head = head.to_lowercase();
                if head.contains("command") || head.contains("vm_exec") || head.contains("shell") {
                    after
                } else {
                    rest
                }
            }
            None => rest,
        };
        let lower = candidate.to_lowercase();
        let cut = [
            ". report",
            ". show",
            ". do not",
            "; report",
            "; show",
            " and report ",
            " and show ",
            " then ",
            " do not ",
        ]
        .iter()
        .filter_map(|needle| lower.find(needle))
        .min()
        .unwrap_or(candidate.len());
        candidate[..cut]
            .trim_matches(|c: char| c == '"' || c == '\'' || c == '.' || c == ';')
            .trim()
            .to_string()
    };
    if command.is_empty() || command.len() > 16_000 {
        return None;
    }
    Some(json!({
        "command": command,
        "cwd": "/opt/vibe/app",
        "reason": "requested in chat",
    }))
}
