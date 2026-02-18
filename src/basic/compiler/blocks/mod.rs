pub mod mail;
pub mod talk;

pub use mail::convert_mail_block;
pub use talk::convert_talk_block;

use log::info;

pub fn convert_begin_blocks(script: &str) -> String {
    let mut result = String::new();
    let mut in_talk_block = false;
    let mut talk_block_lines: Vec<String> = Vec::new();
    let mut in_mail_block = false;
    let mut mail_recipient = String::new();
    let mut mail_block_lines: Vec<String> = Vec::new();

    for line in script.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
            continue;
        }

        if upper == "BEGIN TALK" {
            info!("[TOOL] Converting BEGIN TALK statement");
            in_talk_block = true;
            talk_block_lines.clear();
            continue;
        }

        if upper == "END TALK" {
            info!("[TOOL] Converting END TALK statement, processing {} lines", talk_block_lines.len());
            in_talk_block = false;
            let converted = convert_talk_block(&talk_block_lines);
            result.push_str(&converted);
            talk_block_lines.clear();
            continue;
        }

        if in_talk_block {
            talk_block_lines.push(trimmed.to_string());
            continue;
        }

        if upper.starts_with("BEGIN MAIL ") {
            let recipient = &trimmed[11..].trim();
            info!("[TOOL] Converting BEGIN MAIL statement: recipient='{}'", recipient);
            mail_recipient = recipient.to_string();
            in_mail_block = true;
            mail_block_lines.clear();
            continue;
        }

        if upper == "END MAIL" {
            info!("[TOOL] Converting END MAIL statement, processing {} lines", mail_block_lines.len());
            in_mail_block = false;
            let converted = convert_mail_block(&mail_recipient, &mail_block_lines);
            result.push_str(&converted);
            result.push('\n');
            mail_recipient.clear();
            mail_block_lines.clear();
            continue;
        }

        if in_mail_block {
            mail_block_lines.push(trimmed.to_string());
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}
