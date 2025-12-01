//! Custom Askama filters for web templates

use askama::Result;

/// Default filter - returns the value if non-empty, otherwise returns the default
pub fn default(value: &str, default_value: &str) -> Result<String> {
    if value.is_empty() {
        Ok(default_value.to_string())
    } else {
        Ok(value.to_string())
    }
}

/// Truncate filter - truncates a string to a maximum length
pub fn truncate(value: &str, max_len: usize) -> Result<String> {
    if value.len() > max_len {
        Ok(format!("{}...", &value[..max_len.saturating_sub(3)]))
    } else {
        Ok(value.to_string())
    }
}

/// Title case filter - capitalizes the first letter of each word
pub fn title(value: &str) -> Result<String> {
    Ok(value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" "))
}

/// Format date filter - formats a timestamp string
pub fn format_date(value: &str, format: &str) -> Result<String> {
    // Simple implementation - in production would use chrono
    if format == "short" {
        Ok(value.chars().take(10).collect())
    } else {
        Ok(value.to_string())
    }
}

/// Pluralize filter - returns singular or plural form based on count
pub fn pluralize(count: i64, singular: &str, plural: &str) -> Result<String> {
    if count == 1 {
        Ok(singular.to_string())
    } else {
        Ok(plural.to_string())
    }
}

/// File size filter - formats bytes as human-readable size
pub fn filesize(bytes: u64) -> Result<String> {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        Ok(format!("{:.1} GB", bytes as f64 / GB as f64))
    } else if bytes >= MB {
        Ok(format!("{:.1} MB", bytes as f64 / MB as f64))
    } else if bytes >= KB {
        Ok(format!("{:.1} KB", bytes as f64 / KB as f64))
    } else {
        Ok(format!("{} B", bytes))
    }
}

/// Initials filter - extracts initials from a name
pub fn initials(name: &str) -> Result<String> {
    Ok(name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase())
}

/// Escape JavaScript filter - escapes string for use in JavaScript
pub fn escapejs(value: &str) -> Result<String> {
    Ok(value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t"))
}

/// JSON filter - converts value to JSON string
pub fn json(value: &str) -> Result<String> {
    Ok(format!("\"{}\"", escapejs(value)?))
}

/// Slugify filter - converts string to URL-safe slug
pub fn slugify(value: &str) -> Result<String> {
    Ok(value
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-"))
}
