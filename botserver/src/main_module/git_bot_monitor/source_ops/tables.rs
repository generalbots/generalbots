use super::*;

/// Split a `tables.bas` body into `(table_name, block)` pairs.
pub(super) fn split_table_blocks(tables_bas: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in tables_bas.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if upper.starts_with("BEGIN TABLE ") {
            let name = trimmed[12..].trim().to_string();
            if !name.is_empty() {
                current = Some((name, vec![line]));
            }
            continue;
        }
        if upper.starts_with("END TABLE") {
            if let Some((name, mut lines)) = current.take() {
                lines.push(line);
                blocks.push((name, lines.join("\n")));
            }
            continue;
        }
        if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    blocks
}

/// Merge the `BEGIN TABLE … END TABLE` blocks of `tables_bas` into the bot's
/// `.gbdialog/tables.bas`, appending only tables that are not declared yet.
/// Returns the table names appended (empty when the schema already covered
/// them), so the caller can report what changed.
pub(crate) fn merge_tables(
    pool: &DbPool,
    bot_name: &str,
    tables_bas: &str,
) -> Result<Vec<String>, String> {
    let blocks = split_table_blocks(tables_bas);
    if blocks.is_empty() {
        return Ok(Vec::new());
    }
    let target = find_source_target(pool, bot_name)?;
    let checkout = prepare_checkout(pool, &target)?;
    let dialog = dialog_dir(&checkout, bot_name)?;
    let path = dialog.join("tables.bas");
    let existing = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let mut body = existing.clone();
    let mut appended = Vec::new();
    for (name, block) in blocks {
        if existing.contains(&format!("BEGIN TABLE {name}")) {
            continue;
        }
        if !body.is_empty() && !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(&block);
        if !body.ends_with('\n') {
            body.push('\n');
        }
        appended.push(name);
    }
    if appended.is_empty() {
        return Ok(Vec::new());
    }
    std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
    commit_and_push(
        pool,
        &checkout,
        &target,
        &format!("autotask: attach tables ({})", appended.join(", ")),
        &["tables.bas".to_string()],
    )?;
    Ok(appended)
}
