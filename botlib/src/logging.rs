use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

fn format_compact_to(record: &Record, w: &mut dyn Write) -> std::io::Result<()> {
    let level_str = match record.level() {
        log::Level::Error => "error",
        log::Level::Warn => "warn",
        log::Level::Info => "info",
        log::Level::Debug => "debug",
        log::Level::Trace => "trace",
    };

    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S%.3f");

    let target = record.target();
    let module = if let Some(pos) = target.rfind("::") {
        &target[pos + 2..]
    } else {
        target
    };

    let prefix = format!("{} {} {}:", timestamp, level_str, module);
    let message = record.args().to_string();
    let indent = "                         ";

    if prefix.len() + message.len() <= 100 {
        writeln!(w, "{}{}", prefix, message)
    } else {
        let available_first = 100usize.saturating_sub(prefix.len());
        let available_other = 75usize;

        write!(w, "{}", prefix)?;
        let chars: Vec<char> = message.chars().collect();
        let total = chars.len();
        let first_take = available_first.min(total);
        let first: String = chars[..first_take].iter().collect();
        writeln!(w, "{}", first)?;

        let mut pos = first_take;
        while pos < total {
            write!(w, "{}", indent)?;
            let take = available_other.min(total - pos);
            let chunk: String = chars[pos..pos + take].iter().collect();
            writeln!(w, "{}", chunk)?;
            pos += take;
        }
        Ok(())
    }
}

/// env_logger formatter: info to stdout, warn/error/debug/trace to stderr.
pub fn compact_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    match record.level() {
        log::Level::Info => {
            let mut w = buf;
            format_compact_to(record, &mut w)
        }
        _ => format_compact_to(record, &mut std::io::stderr().lock()),
    }
}

pub fn init_compact_logger(default_filter: &str) {
    // #1921 — botlib runs as a redirected sidecar in the desktop shell, where
    // stdout is block-buffered and `info!` lines never reach the log file. A
    // LineWriter flushes on every newline so diagnostic output is visible live.
    let stdout = std::io::LineWriter::new(std::io::stdout());
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_filter))
        .target(env_logger::Target::Pipe(Box::new(stdout)))
        .format(compact_format)
        .init();
}

pub fn init_compact_logger_with_style(default_filter: &str) {
    init_compact_logger(default_filter);
}

#[cfg(test)]
mod compact_format_tests {
    use super::*;
    use log::{Level, Record};

    fn render(level: Level, target: &'static str, message: &str) -> String {
        let args = format_args!("{}", message);
        let record = Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .build();
        let mut buffer: Vec<u8> = Vec::new();
        format_compact_to(&record, &mut buffer).expect("format");
        String::from_utf8(buffer).expect("utf8")
    }

    #[test]
    fn a_short_line_is_one_line_with_the_level_and_module() {
        let line = render(Level::Info, "crate::module", "hello");
        assert_eq!(line.lines().count(), 1);
        assert!(line.contains(" info module:"));
        assert!(line.trim_end().ends_with("hello"));
    }

    #[test]
    fn every_level_uses_its_lowercase_word() {
        for (level, word) in [
            (Level::Error, "error"),
            (Level::Warn, "warn"),
            (Level::Info, "info"),
            (Level::Debug, "debug"),
            (Level::Trace, "trace"),
        ] {
            let line = render(level, "crate::module", "x");
            assert!(line.contains(&format!(" {word} module:")), "{line}");
        }
    }

    #[test]
    fn a_target_without_a_module_separator_is_kept_whole() {
        let line = render(Level::Info, "solo", "x");
        assert!(line.contains(" info solo:"));
    }

    #[test]
    fn a_long_line_wraps_with_the_continuation_indent() {
        let message = "a".repeat(300);
        let line = render(Level::Info, "crate::module", &message);
        let lines: Vec<&str> = line.lines().collect();
        assert!(lines.len() > 1, "long messages must wrap");
        for continuation in &lines[1..] {
            assert!(
                continuation.starts_with("                         "),
                "continuation must be indented: {continuation:?}"
            );
        }
        // No character is lost by wrapping.
        let joined: String = lines
            .iter()
            .map(|l| l.trim_start())
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(joined.matches('a').count(), 300);
    }

    #[test]
    fn multibyte_messages_are_split_on_character_boundaries() {
        let message = "ç".repeat(200);
        let line = render(Level::Info, "crate::module", &message);
        assert!(line.chars().all(|c| c == 'ç' || c.is_ascii()));
        assert_eq!(line.matches('ç').count(), 200);
    }
}
