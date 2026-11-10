// Tests for the TABLE ... END TABLE parser and the DDL generator.
// Split into `tests/` because `table_definition` already exceeds the size
// budget of a single file.

use botbasic_data::keywords::table_definition::{
    build_connection_string, generate_create_table_sql, map_type_to_sql, parse_table_definition,
    ExternalConnection, FieldDefinition,
};

fn field(name: &str, field_type: &str) -> FieldDefinition {
    FieldDefinition {
        name: name.to_string(),
        field_type: field_type.to_string(),
        ..Default::default()
    }
}

#[test]
fn parses_a_single_table_with_fields() {
    let source = "\
TABLE customers
    id AS GUID KEY
    name AS STRING(120)
    total AS NUMBER(10,2)
END TABLE";

    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(tables.len(), 1);

    let table = &tables[0];
    assert_eq!(table.name, "customers");
    assert_eq!(table.connection_name, "default");
    assert_eq!(table.fields.len(), 3);
    assert_eq!(table.fields[0].field_type, "guid");
    assert!(table.fields[0].is_key);
    assert!(!table.fields[0].is_nullable);
    assert_eq!(table.fields[1].length, Some(120));
    assert_eq!(table.fields[2].length, Some(10));
    assert_eq!(table.fields[2].precision, Some(2));
    assert!(table.fields[1].is_nullable);
}

#[test]
fn parses_multiple_tables_and_ignores_comments() {
    let source = "\
' a comment
TABLE a
    id AS GUID KEY
END TABLE
REM another comment
TABLE b
    title AS TEXT
END TABLE";

    let tables = parse_table_definition(source).expect("valid tables");
    assert_eq!(tables.len(), 2);
    assert_eq!(tables[0].name, "a");
    assert_eq!(tables[1].name, "b");
}

#[test]
fn table_level_roles_are_parsed_from_quoted_lists() {
    let source = "TABLE orders READ BY \"admin; auditor\" WRITE BY \"admin\"\n    id AS GUID KEY\nEND TABLE";
    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(tables[0].read_roles, vec!["admin", "auditor"]);
    assert_eq!(tables[0].write_roles, vec!["admin"]);
}

#[test]
fn connection_name_is_read_from_the_on_clause() {
    let source = "TABLE orders ON erp\n    id AS GUID KEY\nEND TABLE";
    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(tables[0].connection_name, "erp");
}

#[test]
fn on_read_does_not_become_a_connection_name() {
    let source = "TABLE orders ON READ BY \"admin\"\n    id AS GUID KEY\nEND TABLE";
    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(tables[0].connection_name, "default");
}

#[test]
fn field_references_are_captured() {
    let source = "TABLE orders\n    customer AS GUID REFERENCES customers\nEND TABLE";
    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(
        tables[0].fields[0].reference_table.as_deref(),
        Some("customers")
    );
}

#[test]
fn field_keyword_prefix_is_optional() {
    let source = "TABLE t\n    FIELD code AS STRING(10)\nEND TABLE";
    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(tables[0].fields[0].name, "code");
    assert_eq!(tables[0].fields[0].field_type, "string");
}

#[test]
fn missing_type_defaults_to_text() {
    let source = "TABLE t\n    note\nEND TABLE";
    let tables = parse_table_definition(source).expect("valid table");
    assert_eq!(tables[0].fields[0].field_type, "text");
}

#[test]
fn a_bare_table_keyword_is_not_treated_as_a_header() {
    // `TABLE` with no name is skipped rather than parsed: only `TABLE <name>`
    // opens a definition.
    assert!(parse_table_definition("TABLE\nEND TABLE").expect("ok").is_empty());
}

#[test]
fn source_without_tables_yields_an_empty_list() {
    assert!(parse_table_definition("TALK \"hi\"").expect("ok").is_empty());
}

#[test]
fn map_type_to_sql_covers_every_documented_type() {
    assert_eq!(map_type_to_sql(&field("a", "string"), "postgres"), "VARCHAR(255)");
    assert_eq!(
        map_type_to_sql(
            &FieldDefinition {
                length: Some(50),
                ..field("a", "string")
            },
            "postgres"
        ),
        "VARCHAR(50)"
    );
    assert_eq!(map_type_to_sql(&field("a", "integer"), "postgres"), "INTEGER");
    assert_eq!(
        map_type_to_sql(
            &FieldDefinition {
                length: Some(12),
                ..field("a", "int")
            },
            "postgres"
        ),
        "BIGINT"
    );
    assert_eq!(
        map_type_to_sql(
            &FieldDefinition {
                length: Some(10),
                precision: Some(2),
                ..field("a", "number")
            },
            "postgres"
        ),
        "DECIMAL(10,2)"
    );
    assert_eq!(map_type_to_sql(&field("a", "float"), "postgres"), "DOUBLE PRECISION");
    assert_eq!(map_type_to_sql(&field("a", "date"), "postgres"), "DATE");
    assert_eq!(map_type_to_sql(&field("a", "text"), "postgres"), "TEXT");
}

#[test]
fn map_type_to_sql_is_dialect_aware() {
    assert_eq!(map_type_to_sql(&field("a", "datetime"), "mysql"), "DATETIME");
    assert_eq!(map_type_to_sql(&field("a", "datetime"), "postgres"), "TIMESTAMP");
    assert_eq!(map_type_to_sql(&field("a", "boolean"), "mariadb"), "TINYINT(1)");
    assert_eq!(map_type_to_sql(&field("a", "boolean"), "postgres"), "BOOLEAN");
    assert_eq!(map_type_to_sql(&field("a", "uuid"), "mysql"), "CHAR(36)");
    assert_eq!(map_type_to_sql(&field("a", "guid"), "postgres"), "UUID");
}

#[test]
fn unknown_types_fall_back_to_varchar() {
    assert_eq!(map_type_to_sql(&field("a", "moneyish"), "postgres"), "VARCHAR(255)");
}

#[test]
fn create_table_sql_emits_columns_keys_and_not_null() {
    let source = "\
TABLE customers
    id AS GUID KEY
    name AS STRING(80)
END TABLE";
    let table = &parse_table_definition(source).expect("valid")[0];
    let sql = generate_create_table_sql(table, "postgres");

    assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS customers ("));
    assert!(sql.contains("    id UUID NOT NULL"));
    assert!(sql.contains("    name VARCHAR(80)"));
    assert!(sql.contains("PRIMARY KEY (id)"));
    assert!(sql.ends_with(");"));
}

#[test]
fn create_table_sql_sanitizes_identifiers() {
    let source = "\
TABLE bad;name
    id AS GUID KEY
END TABLE";
    let table = &parse_table_definition(source).expect("valid")[0];
    let sql = generate_create_table_sql(table, "postgres");
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS badname ("));
    assert!(!sql.contains(';') || sql.ends_with(");"));
}

#[test]
fn create_table_sql_adds_mysql_engine_clause() {
    let table = &parse_table_definition("TABLE t\n    id AS GUID KEY\nEND TABLE")
        .expect("valid")[0];
    let sql = generate_create_table_sql(table, "mysql");
    assert!(sql.contains("ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"));
    assert!(sql.ends_with(';'));
}

#[test]
fn create_table_sql_without_keys_has_no_primary_key_clause() {
    let table = &parse_table_definition("TABLE t\n    name AS TEXT\nEND TABLE").expect("valid")[0];
    let sql = generate_create_table_sql(table, "postgres");
    assert!(!sql.contains("PRIMARY KEY"));
}

#[test]
fn connection_strings_use_the_driver_default_port() {
    let conn = |driver: &str| ExternalConnection {
        name: "c".into(),
        driver: driver.into(),
        server: "db.local".into(),
        port: None,
        database: "sales".into(),
        username: "u".into(),
        password: "p".into(),
    };

    assert_eq!(
        build_connection_string(&conn("postgres")),
        "postgres://u:p@db.local:5432/sales"
    );
    assert_eq!(
        build_connection_string(&conn("mysql")),
        "mysql://u:p@db.local:3306/sales"
    );
    assert_eq!(
        build_connection_string(&conn("mssql")),
        "mssql://u:p@db.local:1433/sales"
    );
}

#[test]
fn connection_string_honours_an_explicit_port() {
    let conn = ExternalConnection {
        name: "c".into(),
        driver: "postgres".into(),
        server: "db.local".into(),
        port: Some(6543),
        database: "sales".into(),
        username: "u".into(),
        password: "p".into(),
    };
    assert_eq!(
        build_connection_string(&conn),
        "postgres://u:p@db.local:6543/sales"
    );
}
