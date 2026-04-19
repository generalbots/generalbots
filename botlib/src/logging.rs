use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

pub fn compact_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
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

    // Format: "YYYYMMDDHHMMSS.mmm level module:"
    // Length: 18 + 1 + 5 (error) + 1 + module.len() + 1 = 26 + module.len()
    let prefix = format!("{} {} {}:", timestamp, level_str, module);

    // Max width 100
    // Indent for wrapping: 18 timestamp + 1 space + 5 (longest level "error") + 1 space = 25 spaces
    let message = record.args().to_string();
    let indent = "                         "; // 25 spaces

    if prefix.len() + message.len() <= 100 {
        writeln!(buf, "{}{}", prefix, message)
    } else {
        let available_first_line = if prefix.len() < 100 {
            100 - prefix.len()
        } else {
            0
        };
        let available_other_lines = 100 - 25; // 75 chars

        let mut current_pos = 0;
        let chars: Vec<char> = message.chars().collect();
        let total_chars = chars.len();

        // First line
        write!(buf, "{}", prefix)?;

        // If prefix is already >= 100, we force a newline immediately?
        // Or we just print a bit and wrap?
        // Let's assume typical usage where module name isn't huge.

        let take = std::cmp::min(available_first_line, total_chars);
        let first_chunk: String = chars[0..take].iter().collect();
        writeln!(buf, "{}", first_chunk)?;
        current_pos += take;

        while current_pos < total_chars {
            write!(buf, "{}", indent)?;
            let remaining = total_chars - current_pos;
            let take = std::cmp::min(remaining, available_other_lines);
            let chunk: String = chars[current_pos..current_pos + take].iter().collect();
            writeln!(buf, "{}", chunk)?;
            current_pos += take;
        }
        Ok(())
    }
}

pub fn init_compact_logger(default_filter: &str) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_filter))
        .format(compact_format)
        .init();
}

pub fn init_compact_logger_with_style(default_filter: &str) {
    // Style ignored to strictly follow text format spec
    init_compact_logger(default_filter);
}
