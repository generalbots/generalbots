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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_filter))
        .target(env_logger::Target::Stdout)
        .format(compact_format)
        .init();
}

pub fn init_compact_logger_with_style(default_filter: &str) {
    init_compact_logger(default_filter);
}
