//! Reform #1505 — `tables.bas` material produced by AutoTask.
//!
//! A generated automation that needs storage declares its schema with
//! `BEGIN TABLE … END TABLE` blocks. `tables.bas` is a *special* bot file (it is
//! parsed at compile time to build the bot's database schema and must never be
//! invoked with `CALL`), so those blocks belong there — not in the tool script.
//! These helpers split a generated body into its executable tool source and its
//! table declarations, and render a manifest `TableDefinition` back to BASIC.

use crate::task_manifest::{FieldDefinition, TableDefinition};

/// Split a generated BASIC body into `(tool_source, tables_source)`.
///
/// The `BEGIN TABLE … END TABLE` blocks are removed from the tool source and
/// returned separately; a body without table declarations returns an empty
/// `tables_source`.
pub fn split_table_blocks(body: &str) -> (String, String) {
    let mut tool = String::new();
    let mut tables = String::new();
    let mut inside_table = false;
    for line in body.lines() {
        let upper = line.trim().to_uppercase();
        if !inside_table && upper.starts_with("BEGIN TABLE ") {
            inside_table = true;
            tables.push_str(line);
            tables.push('\n');
            continue;
        }
        if inside_table {
            tables.push_str(line);
            tables.push('\n');
            if upper.starts_with("END TABLE") {
                inside_table = false;
            }
            continue;
        }
        tool.push_str(line);
        tool.push('\n');
    }
    // An unterminated block means the generation was truncated: keep it with
    // the tool so the failure surfaces instead of silently dropping the schema.
    if inside_table {
        tool.push_str(&tables);
        return (tool, String::new());
    }
    (tool, tables)
}

/// Render a manifest table definition as a `BEGIN TABLE … END TABLE` block.
#[must_use]
pub fn render_table(table: &TableDefinition) -> String {
    let mut out = format!("BEGIN TABLE {}\n", table.name);
    for field in &table.fields {
        out.push_str(&render_field(field));
        out.push('\n');
    }
    out.push_str("END TABLE\n");
    out
}

fn render_field(field: &FieldDefinition) -> String {
    let mut line = format!("    {} {}", field.name, field.field_type);
    if field.is_key {
        line.push_str(" PRIMARY KEY");
    }
    if field.is_nullable || field.nullable {
        // BASIC schema is nullable by default; only flagged fields are noted.
        line.push_str(" NULL");
    }
    if let Some(reference) = &field.reference_table {
        line.push_str(&format!(" REFERENCES {reference}"));
    }
    if let Some(default) = &field.default_value {
        line.push_str(&format!(" DEFAULT {default}"));
    }
    line
}

impl TableDefinition {
    /// This table as a `tables.bas` fragment.
    #[must_use]
    pub fn to_basic(&self) -> String {
        render_table(self)
    }
}

#[cfg(test)]
mod tests {
    use super::split_table_blocks;

    #[test]
    fn pulls_tables_out_of_the_tool_source() {
        let body = "TALK \"hi\"\nBEGIN TABLE orders\n    id UUID PRIMARY KEY\nEND TABLE\nSAVE o TO orders\n";
        let (tool, tables) = split_table_blocks(body);
        assert!(tool.contains("TALK \"hi\""));
        assert!(tool.contains("SAVE o TO orders"));
        assert!(!tool.contains("BEGIN TABLE"));
        assert!(tables.contains("BEGIN TABLE orders"));
        assert!(tables.contains("END TABLE"));
    }

    #[test]
    fn keeps_an_unterminated_block_with_the_tool() {
        let body = "BEGIN TABLE broken\n    id UUID\n";
        let (tool, tables) = split_table_blocks(body);
        assert!(tables.is_empty());
        assert!(tool.contains("BEGIN TABLE broken"));
    }

    #[test]
    fn plain_scripts_are_untouched() {
        let (tool, tables) = split_table_blocks("TALK \"ok\"\n");
        assert_eq!(tool, "TALK \"ok\"\n");
        assert!(tables.is_empty());
    }
}
