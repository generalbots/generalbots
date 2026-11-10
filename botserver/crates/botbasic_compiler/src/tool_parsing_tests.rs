// Tests for the tool-manifest parser (`PARAM` / `DESCRIPTION` / `ANNOTATION`)
// and the MCP/OpenAI schema generators. Split out of `tool_parsing.rs` to keep
// that file under the AGENTS.md size limit.

use crate::types::*;
use crate::BasicCompiler;

fn param(line: &str) -> ParamDeclaration {
    BasicCompiler::parse_param_line(line)
        .expect("parsed")
        .expect("a declaration")
}

#[test]
fn normalize_type_maps_every_alias() {
    for (raw, expected) in [
        ("integer", "integer"),
        ("int", "integer"),
        ("number", "integer"),
        ("float", "number"),
        ("double", "number"),
        ("decimal", "number"),
        ("boolean", "boolean"),
        ("bool", "boolean"),
        ("array", "array"),
        ("list", "array"),
        ("object", "object"),
        ("map", "object"),
        ("date", "string"),
        ("text", "string"),
        ("", "string"),
    ] {
        assert_eq!(BasicCompiler::normalize_type(raw), expected, "type {raw}");
    }
    assert_eq!(BasicCompiler::normalize_type("INT"), "integer");
}

#[test]
fn param_line_requires_the_param_keyword() {
    assert!(BasicCompiler::parse_param_line("DESCRIPTION \"x\"")
        .expect("parsed")
        .is_none());
    assert!(BasicCompiler::parse_param_line("PARAM")
        .expect("parsed")
        .is_none());
}

#[test]
fn param_line_parses_name_type_and_description() {
    let parsed = param("PARAM city:string DESCRIPTION \"The city\"");
    assert_eq!(parsed.name, "city");
    assert_eq!(parsed.param_type, "string");
    assert_eq!(parsed.original_type, "string");
    assert_eq!(parsed.description, "The city");
    assert!(parsed.required);
    assert_eq!(parsed.example, None);
    assert_eq!(parsed.enum_values, None);
}

#[test]
fn param_line_defaults_the_type_to_string() {
    let parsed = param("PARAM q DESCRIPTION \"Query\"");
    assert_eq!(parsed.name, "q");
    assert_eq!(parsed.param_type, "string");
}

#[test]
fn param_line_accepts_the_as_type_form() {
    let parsed = param("PARAM q AS text LIKE \"hello\" DESCRIPTION \"Query\"");
    assert_eq!(parsed.original_type, "text");
    assert_eq!(parsed.param_type, "string");
    assert_eq!(parsed.example, Some("hello".to_string()));
}

#[test]
fn param_line_keeps_the_original_type_for_the_date_format() {
    let parsed = param("PARAM day:date DESCRIPTION \"Day\"");
    assert_eq!(parsed.original_type, "date");
    assert_eq!(parsed.param_type, "string");
}

#[test]
fn param_line_parses_an_enum_list() {
    let parsed = param("PARAM mode:string ENUM [\"fast\", \"slow\"] DESCRIPTION \"Speed\"");
    assert_eq!(
        parsed.enum_values,
        Some(vec!["fast".to_string(), "slow".to_string()])
    );
}

#[test]
fn param_line_detects_optional_parameters() {
    let optional = param("PARAM days:integer DESCRIPTION \"Days (optional)\"");
    assert!(!optional.required);
    let optional_pt = param("PARAM days:integer DESCRIPTION \"Dias (opcional)\"");
    assert!(!optional_pt.required);
    let required = param("PARAM days:integer DESCRIPTION \"Days\"");
    assert!(required.required);
}

#[test]
fn annotation_line_accepts_the_known_hints() {
    for hint in ["READ_ONLY", "DESTRUCTIVE", "IDEMPOTENT", "OPEN_WORLD"] {
        assert_eq!(
            BasicCompiler::parse_annotation_line(&format!("ANNOTATION {hint}")),
            vec![hint.to_string()]
        );
        // Hints are normalised to upper case.
        assert_eq!(
            BasicCompiler::parse_annotation_line(&format!("ANNOTATION {}", hint.to_lowercase())),
            vec![hint.to_string()]
        );
    }
}

#[test]
fn annotation_line_parses_a_title() {
    assert_eq!(
        BasicCompiler::parse_annotation_line("ANNOTATION TITLE \"Deploy target\""),
        vec!["TITLE \"Deploy target\"".to_string()]
    );
    // A title without a quoted value carries nothing.
    assert!(BasicCompiler::parse_annotation_line("ANNOTATION TITLE").is_empty());
}

#[test]
fn annotation_line_ignores_unknown_hints_and_other_lines() {
    assert!(BasicCompiler::parse_annotation_line("ANNOTATION WHATEVER").is_empty());
    assert!(BasicCompiler::parse_annotation_line("TALK \"x\"").is_empty());
}

#[test]
fn mcp_annotations_are_absent_when_nothing_is_declared() {
    let def = ToolDefinition {
        name: "t".to_string(),
        description: String::new(),
        parameters: Vec::new(),
        source_file: "t.bas".to_string(),
        annotations: Vec::new(),
    };
    assert!(BasicCompiler::generate_mcp_annotations(&def).is_none());
}

#[test]
fn mcp_annotations_record_the_declared_hints() {
    let def = ToolDefinition {
        name: "t".to_string(),
        description: String::new(),
        parameters: Vec::new(),
        source_file: "t.bas".to_string(),
        annotations: vec![
            "READ_ONLY".to_string(),
            "TITLE \"Deploy target\"".to_string(),
        ],
    };
    let annotations = BasicCompiler::generate_mcp_annotations(&def).expect("annotations");
    assert_eq!(annotations.title, Some("Deploy target".to_string()));
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, None);
    assert_eq!(annotations.idempotent_hint, None);
    assert_eq!(annotations.open_world_hint, None);
}

#[test]
fn mcp_tool_schema_lists_properties_and_required_parameters() {
    let def = ToolDefinition {
        name: "forecast".to_string(),
        description: "Fetch a forecast".to_string(),
        parameters: vec![
            param("PARAM city:string DESCRIPTION \"The city\""),
            param("PARAM day:date DESCRIPTION \"Day (optional)\""),
        ],
        source_file: "forecast.bas".to_string(),
        annotations: Vec::new(),
    };
    let tool = BasicCompiler::generate_mcp_tool(&def).expect("tool");
    assert_eq!(tool.name, "forecast");
    assert_eq!(tool.input_schema.schema_type, "object");
    assert_eq!(tool.input_schema.required, vec!["city".to_string()]);
    assert_eq!(
        tool.input_schema.properties["city"].prop_type,
        "string"
    );
    assert_eq!(
        tool.input_schema.properties["day"].format,
        Some("date".to_string())
    );
    assert!(tool.annotations.is_none());
}

#[test]
fn openai_tool_schema_carries_the_enum_values() {
    let def = ToolDefinition {
        name: "run".to_string(),
        description: "Run it".to_string(),
        parameters: vec![param(
            "PARAM mode:string ENUM [\"fast\", \"slow\"] DESCRIPTION \"Speed\"",
        )],
        source_file: "run.bas".to_string(),
        annotations: Vec::new(),
    };
    let tool = BasicCompiler::generate_openai_tool(&def).expect("tool");
    assert_eq!(tool.tool_type, "function");
    assert_eq!(tool.function.name, "run");
    assert_eq!(tool.function.parameters.param_type, "object");
    assert_eq!(
        tool.function.parameters.properties["mode"].enum_values,
        Some(vec!["fast".to_string(), "slow".to_string()])
    );
    assert_eq!(tool.function.parameters.required, vec!["mode".to_string()]);
}

const TOOL_SOURCE: &str = "DESCRIPTION \"Fetch a city forecast\"\nPARAM city:string DESCRIPTION \"The city\"\nPARAM days:integer DESCRIPTION \"Days (optional)\"\nANNOTATION READ_ONLY\n\nTALK \"ignored\"\n";

#[test]
fn tool_definition_takes_its_name_from_the_file_stem() {
    let def = BasicCompiler::parse_tool_definition(TOOL_SOURCE, "tools/forecast.bas").expect("parsed");
    assert_eq!(def.name, "forecast");
    assert_eq!(def.source_file, "tools/forecast.bas");
    assert_eq!(def.description, "Fetch a city forecast");
    assert_eq!(def.parameters.len(), 2);
    assert_eq!(def.annotations, vec!["READ_ONLY".to_string()]);
}

#[test]
fn tool_definition_without_a_path_is_named_unknown() {
    let def = BasicCompiler::parse_tool_definition("", "").expect("parsed");
    assert_eq!(def.name, "unknown");
    assert!(def.parameters.is_empty());
    assert!(def.description.is_empty());
}

#[test]
fn manifest_json_serialises_the_mcp_tool() {
    let json = BasicCompiler::manifest_json(TOOL_SOURCE, "forecast.bas").expect("manifest");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(value["name"], "forecast");
    assert_eq!(value["description"], "Fetch a city forecast");
    assert_eq!(value["input_schema"]["type"], "object");
    assert_eq!(value["input_schema"]["properties"]["city"]["type"], "string");
    assert_eq!(value["input_schema"]["required"][0], "city");
    assert_eq!(value["annotations"]["readOnlyHint"], true);
}
