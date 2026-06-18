use super::BasicCompiler;
use botcore::shared::state::AppState;
use diesel::{ExpressionMethods, QueryDsl, QueryableByName, RunQueryDsl};
use log::trace;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;

impl BasicCompiler {
    pub(crate) fn convert_save_statements(
        &self,
        source: &str,
        bot_id: uuid::Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut result = String::new();
        let mut save_counter = 0;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.to_uppercase().starts_with("SAVE ") {
                if let Some(converted) = self.convert_save_line(line, bot_id, &mut save_counter)? {
                    result.push_str(&converted);
                    result.push('\n');
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        Ok(result)
    }

    fn convert_save_line(
        &self,
        line: &str,
        bot_id: uuid::Uuid,
        save_counter: &mut usize,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if !upper.starts_with("SAVE ") {
            return Ok(None);
        }

        let content = &trimmed[4..].trim();

        // Handle SAVE ... TO ... WHERE ... pattern
        // Patterns: SAVE variable TO table WHERE id = expr
        //           SAVE (key = val, ...) TO table WHERE id = expr
        if upper.contains(" WHERE ") && upper.contains(" TO ") {
            return self.convert_save_with_where(content, save_counter);
        }

        let parts = self.parse_save_statement(content)?;

        if parts.len() <= 2 {
            return Ok(None);
        }

        let table_name = &parts[0];
        let table_name = table_name.trim_matches('"');

        log::trace!(
            "Converting SAVE for table: '{}' (original: '{}')",
            table_name,
            &parts[0]
        );

        let column_names = self.get_table_columns_for_save(table_name, bot_id)?;

        let values: Vec<&String> = parts.iter().skip(1).collect();
        let mut map_pairs = Vec::new();

        log::trace!(
            "Matching {} variables to {} columns",
            values.len(),
            column_names.len()
        );

        for value_var in values.iter() {
            let value_lower = value_var.to_lowercase();

            if let Some(column_name) = column_names
                .iter()
                .find(|col| col.to_lowercase() == value_lower)
            {
                map_pairs.push(format!("{}: {}", column_name, value_var));
            } else {
                log::warn!("No matching column for variable '{}'", value_var);
            }
        }

        let map_expr = format!("#{{{}}}", map_pairs.join(", "));
        let data_var = format!("__save_data_{}__", save_counter);
        *save_counter += 1;

        let converted = format!(
            "let {} = {}; SAVE \"{}\", {}",
            data_var, map_expr, table_name, data_var
        );

        Ok(Some(converted))
    }

    fn parse_save_statement(
        &self,
        content: &str,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    current.push('"');
                    chars.next();
                }
                '"' => {
                    in_quotes = !in_quotes;
                    current.push('"');
                }
                ',' if !in_quotes => {
                    parts.push(current.trim().to_string());
                    current = String::new();
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        Ok(parts)
    }

    /// Convert SAVE ... TO ... WHERE ... patterns to Rhai SAVE syntax
    ///
    /// Handles:
    ///   SAVE (key = val, ...) TO table WHERE id = expr
    ///     → let __save_data_N__ = #{key: val, ...}; SAVE "table", expr, __save_data_N__
    ///
    ///   SAVE variable TO table WHERE id = expr
    ///     → SAVE "table", expr, variable
    fn convert_save_with_where(
        &self,
        content: &str,
        save_counter: &mut usize,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let upper = content.to_uppercase();

        let to_pos = upper.find(" TO ").ok_or("No TO found")?;
        let where_pos = upper.find(" WHERE ").ok_or("No WHERE found")?;

        let data_part = content[..to_pos].trim();
        let table_part = content[to_pos + 4..where_pos].trim();
        let condition_part = content[where_pos + 7..].trim();

        let table_name = table_part.trim_matches('"');

        // Extract id expression from condition (supports "id = expr" or "id=expr")
        let cond_upper = condition_part.to_uppercase();
        let id_expr = if cond_upper.starts_with("ID = ") || cond_upper.starts_with("ID=") {
            let eq_pos = condition_part.find('=').unwrap_or(0);
            condition_part[eq_pos + 1..].trim().to_string()
        } else {
            condition_part.to_string()
        };

        if data_part.starts_with('(') && data_part.ends_with(')') {
            // Parenthesized map: (key1 = val1, key2 = val2)
            // Split by commas respecting nested parens
            let inner = &data_part[1..data_part.len() - 1];
            let items = split_by_commas_outside_parens(inner);
            let rhai_items: Vec<String> = items
                .iter()
                .map(|item| {
                    let eq_pos = item.find('=').unwrap_or(0);
                    let key = item[..eq_pos].trim();
                    let val = item[eq_pos + 1..].trim();
                    format!("{}: {}", key, val)
                })
                .collect();

            let data_var = format!("__save_data_{}__", *save_counter);
            *save_counter += 1;

            // Embed id in data map so Rhai's 2-arg SAVE handler can upsert
            // Format: let __save_data_0__ = #{id: id_expr, key: val, ...}; SAVE "table", __save_data_0__
            let all_items = format!("id: {}, {}", id_expr, rhai_items.join(", "));

            Ok(Some(format!(
                "let {} = #{{{}}}; SAVE \"{}\", {}",
                data_var, all_items, table_name, data_var
            )))
        } else {
            // Variable name: SAVE var TO table WHERE id = expr
            // Pass directly to 2-arg SAVE; if var already has an `id` field,
            // handler will upsert, otherwise it inserts
            Ok(Some(format!(
                "SAVE \"{}\", {}",
                table_name, data_part
            )))
        }
    }

    fn get_table_columns_for_save(
        &self,
        table_name: &str,
        bot_id: uuid::Uuid,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        if let Ok(columns) = self.get_columns_from_table_definition(table_name, bot_id) {
            if !columns.is_empty() {
                log::trace!(
                    "Using TABLE definition for '{}': {} columns",
                    table_name,
                    columns.len()
                );
                return Ok(columns);
            }
        }

        self.get_columns_from_database_schema(table_name, bot_id)
    }

    fn get_columns_from_table_definition(
        &self,
        table_name: &str,
        bot_id: uuid::Uuid,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let bot_name = self.get_bot_name_by_id(bot_id)?;
        let work_path = botcore::shared::utils::get_work_path();
        let tables_path = format!(
            "{}/{}.gbai/{}.gbdialog/tables.bas",
            work_path, bot_name, bot_name
        );

        let tables_content = fs::read_to_string(&tables_path)?;
        let columns = self.parse_table_definition_for_fields(&tables_content, table_name)?;

        Ok(columns)
    }

    fn parse_table_definition_for_fields(
        &self,
        content: &str,
        table_name: &str,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut columns = Vec::new();
        let mut in_target_table = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("TABLE ") && trimmed.contains(table_name) {
                in_target_table = true;
                continue;
            }

            if in_target_table {
                if trimmed.starts_with("END TABLE") {
                    break;
                }

                if trimmed.starts_with("FIELD ") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let field_name = parts[1].to_string();
                        columns.push(field_name);
                    }
                }
            }
        }

        Ok(columns)
    }

    pub fn process_tables_bas(
        &self,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bot_name = Self::get_bot_name_from_state(&self.state, self.bot_id)?;
        let work_path = botcore::shared::utils::get_work_path();
        let tables_path = format!(
            "{}/{}.gbai/{}.gbdialog/tables.bas",
            work_path, bot_name, bot_name
        );

        if !Path::new(&tables_path).exists() {
            trace!("tables.bas not found for bot {}, skipping", bot_name);
            return Ok(());
        }

        let tables_content = fs::read_to_string(&tables_path)?;

        trace!(
            "Processing tables.bas for bot {}: {}",
            bot_name,
            tables_path
        );

        if let Some(ref cb) = self.callbacks.process_table_definitions {
            if let Some(ref create_runtime) = self.callbacks.create_runtime {
                let runtime = (create_runtime)(Arc::clone(&self.state));
                (cb)(runtime, self.bot_id, &tables_content)?;
            }
        }

        log::info!("Successfully processed tables.bas for bot {}", bot_name);
        Ok(())
    }

    pub(crate) fn get_bot_name_from_state(
        state: &Arc<AppState>,
        bot_id: uuid::Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut conn = state.conn.get()?;
        use botcore::shared::models::schema::bots::dsl::*;

        bots.filter(id.eq(bot_id))
            .select(name)
            .first::<String>(&mut *conn)
            .map_err(|e| format!("Failed to get bot name: {}", e).into())
    }

    fn get_bot_name_by_id(
        &self,
        bot_id: uuid::Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        use botcore::shared::models::schema::bots::dsl::*;
        use diesel::QueryDsl;

        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let bot_name: String = bots
            .filter(id.eq(&bot_id))
            .select(name)
            .first(&mut conn)
            .map_err(|e| format!("Failed to get bot name: {}", e))?;

        Ok(bot_name)
    }

    fn get_columns_from_database_schema(
        &self,
        table_name: &str,
        bot_id: uuid::Uuid,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        use diesel::sql_query;
        use diesel::sql_types::Text;

        #[derive(QueryableByName)]
        struct ColumnRow {
            #[diesel(sql_type = Text)]
            column_name: String,
        }

        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let query = format!(
            "SELECT column_name FROM information_schema.columns \
             WHERE table_name = '{}' AND table_schema = 'public' \
             ORDER BY ordinal_position",
            table_name
        );

        let columns: Vec<String> = match sql_query(&query).load(&mut conn) {
            Ok(cols) => {
                if cols.is_empty() {
                    log::warn!(
                        "Found 0 columns for table '{}' in main database, trying bot database",
                        table_name
                    );
                    let bot_pool = self.state.bot_database_manager.get_bot_pool(bot_id);
                    if let Some(pool) = bot_pool {
                        let mut bot_conn = pool.get().map_err(|e| format!("Bot DB error: {}", e))?;

                        let bot_query = format!(
                            "SELECT column_name FROM information_schema.columns \
                             WHERE table_name = '{}' AND table_schema = 'public' \
                             ORDER BY ordinal_position",
                            table_name
                        );

                        match sql_query(&bot_query).load(&mut *bot_conn) {
                            Ok(bot_cols) => {
                                log::trace!(
                                    "Found {} columns for table '{}' in bot database",
                                    bot_cols.len(),
                                    table_name
                                );
                                bot_cols
                                    .into_iter()
                                    .map(|c: ColumnRow| c.column_name)
                                    .collect()
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to get columns from bot DB for '{}': {}",
                                    table_name,
                                    e
                                );
                                Vec::new()
                            }
                        }
                    } else {
                        log::error!("No bot database available for bot_id: {}", bot_id);
                        Vec::new()
                    }
                } else {
                    log::trace!(
                        "Found {} columns for table '{}' in main database",
                        cols.len(),
                        table_name
                    );
                    cols.into_iter().map(|c: ColumnRow| c.column_name).collect()
                }
            }
            Err(e) => {
                log::warn!(
                    "Failed to get columns for table '{}' from main DB: {}",
                    table_name,
                    e
                );

                let bot_pool = self.state.bot_database_manager.get_bot_pool(bot_id);
                if let Some(pool) = bot_pool {
                    let mut bot_conn = pool.get().map_err(|e| format!("Bot DB error: {}", e))?;

                    let bot_query = format!(
                        "SELECT column_name FROM information_schema.columns \
                         WHERE table_name = '{}' AND table_schema = 'public' \
                         ORDER BY ordinal_position",
                        table_name
                    );

                    match sql_query(&bot_query).load(&mut *bot_conn) {
                        Ok(cols) => {
                            log::trace!(
                                "Found {} columns for table '{}' in bot database",
                                cols.len(),
                                table_name
                            );
                            cols.into_iter()
                                .filter(|c: &ColumnRow| c.column_name != "id")
                                .map(|c: ColumnRow| c.column_name)
                                .collect()
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to get columns from bot DB for '{}': {}",
                                table_name,
                                e
                            );
                            Vec::new()
                        }
                    }
                } else {
                    log::error!("No bot database available for bot_id: {}", bot_id);
                    Vec::new()
                }
            }
        };

        Ok(columns)
    }
}

/// Split a string by commas that are not inside parentheses.
fn split_by_commas_outside_parens(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;

    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < s.len() {
        parts.push(s[start..].trim());
    }

    parts
}
