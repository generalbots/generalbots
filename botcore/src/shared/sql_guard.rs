pub fn sanitize_identifier(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect()
}

pub fn validate_table_name(name: &str) -> Result<(), String> {
    let sanitized = sanitize_identifier(name);
    if sanitized != name {
        return Err(format!("Invalid table name: {}", name));
    }
    if name.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }
    Ok(())
}

pub fn sanitize_sql_value(value: &str) -> String {
    value.replace('\'', "''").replace(';', "")
}
