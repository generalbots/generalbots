//! What a log line may say about a tool call's arguments. They can carry
//! personal data (a command, an address's query string), so a log names only
//! their size and, for a page fetch, its site.

use crate::core::bot::pipeline::native_tools::web_fetch;

/// `args_bytes=N`, plus ` site=host` for a `web_fetch` call.
pub(crate) fn args_summary(tool: &str, args: &str) -> String {
    let site = if tool == web_fetch::TOOL {
        web_fetch::site_in_args(args)
            .map(|site| format!(" site={site}"))
            .unwrap_or_else(|| { log::debug!("using default for tool site"); Default::default() })
    } else {
        String::new()
    };
    format!("args_bytes={}{site}", args.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_fetch_logs_its_site_and_never_the_query() {
        let args = r#"{"url":"https://www.example.org/busca?cpf=12345678900"}"#;
        let line = args_summary(web_fetch::TOOL, args);
        assert_eq!(line, format!("args_bytes={} site=example.org", args.len()));
        assert!(!line.contains("cpf"));
    }

    #[test]
    fn other_tools_log_only_the_size() {
        let args = r#"{"command":"cat /home/user/notas.txt"}"#;
        assert_eq!(args_summary("terminal_run", args), format!("args_bytes={}", args.len()));
    }
}
