//! Live-database schema compared against `botschema` (#2117).
//!
//! `botschema` is the Diesel source of truth for every table and column; the
//! migrations are the SQL that must produce exactly that shape. A `table!` block
//! no migration creates — or a column the migrations never add — only surfaces
//! later as a runtime Diesel error, so the suite compares the two directly.
//!
//! The `table!` parser below is intentionally line based: `botschema` keeps one
//! declaration per line (`col -> Type,`), which makes a full Rust parser
//! unnecessary and keeps this helper dependency free.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use diesel::prelude::*;
use diesel::sql_types::Text;

/// One column as declared in `botschema` (or as found in the live database).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// PostgreSQL `data_type`, already mapped from the Diesel type. Empty when
    /// the Diesel type is not in the mapping table and therefore not asserted.
    pub sql_type: String,
    pub nullable: bool,
    pub pk: bool,
}

pub type Tables = BTreeMap<String, BTreeMap<String, Column>>;

/// Result of comparing the declaration with the migrated database.
#[derive(Debug, Default)]
pub struct Drift {
    /// Breakage: Diesel would emit a query for something that is not there.
    pub fatal: Vec<String>,
    /// Additive-only differences: extra columns/tables the database has and
    /// `botschema` does not model. Diesel never selects them, so they are
    /// reported (visible in the log) instead of failing the suite.
    pub additive: Vec<String>,
}

/// Maps a Diesel SQL type to the `information_schema.columns.data_type` value.
fn map_type(rust_type: &str) -> &'static str {
    if rust_type.starts_with("Array<") {
        return "ARRAY";
    }
    match rust_type {
        "Uuid" => "uuid",
        "Text" | "Citext" => "text",
        "Varchar" | "Bpchar" => "character varying",
        "Bool" => "boolean",
        "Timestamptz" => "timestamp with time zone",
        "Timestamp" => "timestamp without time zone",
        "Date" => "date",
        "Time" => "time without time zone",
        "Int2" => "smallint",
        "Int4" => "integer",
        "Int8" | "BigInt" => "bigint",
        "Float4" => "real",
        "Float8" | "Double" => "double precision",
        "Numeric" | "Decimal" => "numeric",
        "Jsonb" => "jsonb",
        "Json" => "json",
        "Bytea" => "bytea",
        "Inet" => "inet",
        "Money" => "money",
        _ => "",
    }
}

/// Splits `Nullable<Inner>`; anything else is `NOT NULL`.
fn unwrap_nullable(rust_type: &str) -> (&str, bool) {
    match rust_type
        .strip_prefix("Nullable<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        Some(inner) => (inner, true),
        None => (rust_type, false),
    }
}

fn parse_table_header(line: &str) -> Option<(String, Vec<String>)> {
    if !line.ends_with('{') {
        return None;
    }
    let open = line.find('(')?;
    let close = line.find(')')?;
    if close < open {
        return None;
    }
    let name_part = line.get(..open)?.trim();
    let is_identifier = !name_part.is_empty()
        && name_part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    if !is_identifier {
        return None;
    }
    let name = match name_part.rsplit('.').next() {
        Some(tail) => tail.to_string(),
        None => name_part.to_string(),
    };
    // Composite primary keys are a comma-separated list: `t (a, b) {`.
    let pk = line
        .get(open + 1..close)?
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect();
    Some((name, pk))
}

fn parse_column(raw: &str) -> Option<(String, String)> {
    let (left, right) = raw.split_once("->")?;
    let name = left.trim().to_string();
    let sql_type = right.trim().trim_end_matches(',').trim().to_string();
    if name.is_empty() || sql_type.is_empty() {
        return None;
    }
    Some((name, sql_type))
}

fn parse_file(text: &str, tables: &mut Tables) {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        i += 1;
        // Only a `table! {` opener makes the next item a table declaration;
        // matching on the header alone would also catch Rust `for (a, b) in x {`.
        if line.starts_with("//") || !line.ends_with("table! {") {
            continue;
        }
        let mut header = None;
        while i < lines.len() {
            let raw = lines[i].trim();
            i += 1;
            if raw.is_empty() || raw.starts_with("//") || raw.starts_with('#') {
                continue;
            }
            header = parse_table_header(raw);
            break;
        }
        let Some((name, pk)) = header else {
            continue;
        };
        let mut columns: BTreeMap<String, Column> = BTreeMap::new();
        while i < lines.len() {
            let raw = lines[i].trim();
            i += 1;
            if raw.starts_with('}') {
                break;
            }
            if raw.is_empty() || raw.starts_with("//") || raw.starts_with('#') {
                continue;
            }
            let Some((column, rust_type)) = parse_column(raw) else {
                continue;
            };
            let (inner, nullable) = unwrap_nullable(&rust_type);
            columns.insert(
                column,
                Column {
                    sql_type: map_type(inner).to_string(),
                    nullable,
                    pk: false,
                },
            );
        }
        for column in &pk {
            if let Some(declared) = columns.get_mut(column) {
                declared.pk = true;
            }
        }
        tables.insert(name, columns);
    }
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// Every table/column declared under `botschema/src`.
pub fn read_botschema(dir: &Path) -> Result<Tables, String> {
    let mut files = Vec::new();
    collect_rust_files(dir, &mut files)?;
    files.sort();
    let mut tables = Tables::new();
    for path in files {
        let text =
            fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        parse_file(&text, &mut tables);
    }
    Ok(tables)
}

#[derive(QueryableByName)]
struct ColumnRow {
    #[diesel(sql_type = Text)]
    table_name: String,
    #[diesel(sql_type = Text)]
    column_name: String,
    #[diesel(sql_type = Text)]
    data_type: String,
    #[diesel(sql_type = Text)]
    is_nullable: String,
}

#[derive(QueryableByName)]
struct PrimaryKeyRow {
    #[diesel(sql_type = Text)]
    table_name: String,
    #[diesel(sql_type = Text)]
    column_name: String,
}

/// Every table/column the migrated database actually has, in `public`.
pub fn read_live(conn: &mut PgConnection) -> Result<Tables, String> {
    let rows: Vec<ColumnRow> = diesel::sql_query(
        "SELECT c.table_name, c.column_name, c.data_type, c.is_nullable \
         FROM information_schema.columns c \
         JOIN information_schema.tables t \
           ON t.table_schema = c.table_schema AND t.table_name = c.table_name \
         WHERE c.table_schema = 'public' AND t.table_type = 'BASE TABLE'",
    )
    .load(conn)
    .map_err(|e| format!("information_schema.columns: {e}"))?;

    let mut tables = Tables::new();
    for row in rows {
        tables.entry(row.table_name).or_default().insert(
            row.column_name,
            Column {
                sql_type: row.data_type,
                nullable: row.is_nullable == "YES",
                pk: false,
            },
        );
    }

    let keys: Vec<PrimaryKeyRow> = diesel::sql_query(
        "SELECT tc.table_name, kcu.column_name \
         FROM information_schema.table_constraints tc \
         JOIN information_schema.key_column_usage kcu \
           ON kcu.constraint_name = tc.constraint_name \
          AND kcu.table_schema = tc.table_schema \
         WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = 'public'",
    )
    .load(conn)
    .map_err(|e| format!("information_schema primary keys: {e}"))?;
    for key in keys {
        if let Some(column) = tables
            .get_mut(&key.table_name)
            .and_then(|cols| cols.get_mut(&key.column_name))
        {
            column.pk = true;
        }
    }
    Ok(tables)
}

/// `text` and `varchar` are interchangeable for Diesel (`String` both ways), so
/// a length change is not drift the query layer can break on.
fn same_type(declared: &str, live: &str) -> bool {
    declared == live
        || (declared == "character varying" && live == "text")
        || (declared == "text" && live == "character varying")
}

/// Compares the `botschema` declaration with the live schema.
pub fn compare(expected: &Tables, live: &Tables) -> Drift {
    let mut drift = Drift::default();
    for (table, columns) in expected {
        let Some(actual) = live.get(table) else {
            drift.fatal.push(format!(
                "table `{table}` is declared in botschema but no migration creates it"
            ));
            continue;
        };
        for (column, declared) in columns {
            let Some(found) = actual.get(column) else {
                drift.fatal.push(format!(
                    "column `{table}.{column}` is declared in botschema but no migration creates it"
                ));
                continue;
            };
            if !declared.sql_type.is_empty() && !same_type(&declared.sql_type, &found.sql_type) {
                drift.fatal.push(format!(
                    "column `{table}.{column}` is `{}` in botschema but `{}` in the database",
                    declared.sql_type, found.sql_type
                ));
            }
            if declared.nullable != found.nullable {
                drift.fatal.push(format!(
                    "column `{table}.{column}` is {} in botschema but {} in the database",
                    if declared.nullable { "nullable" } else { "NOT NULL" },
                    if found.nullable { "nullable" } else { "NOT NULL" },
                ));
            }
            if declared.pk != found.pk {
                drift.fatal.push(format!(
                    "column `{table}.{column}` is {} a primary key in botschema but {} in the database",
                    if declared.pk { "" } else { "not" },
                    if found.pk { "is" } else { "is not" },
                ));
            }
        }
        for column in actual.keys() {
            if !columns.contains_key(column) {
                drift.additive.push(format!(
                    "column `{table}.{column}` exists in the database but is not declared in botschema"
                ));
            }
        }
    }
    for table in live.keys() {
        if !expected.contains_key(table) {
            drift.additive.push(format!(
                "table `{table}` exists in the database but is not declared in botschema"
            ));
        }
    }
    drift
}
