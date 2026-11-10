//! Contract test for the `transfer_to_human` tool schema (#2064).
//!
//! The LLM decides whether to escalate based on this JSON schema, so the tool
//! name, the parameter names and the fact that nothing is required are part of
//! the behaviour, not documentation.

use super::get_tool_definition;

#[test]
fn the_tool_schema_is_stable() {
    let definition = get_tool_definition();

    assert_eq!(definition["type"], "function");
    let function = &definition["function"];
    assert_eq!(function["name"], "transfer_to_human");
    assert!(function["description"]
        .as_str()
        .is_some_and(|text| text.contains("human")));

    let properties = &function["parameters"]["properties"];
    for parameter in ["name", "department", "priority", "reason"] {
        assert_eq!(
            properties[parameter]["type"], "string",
            "{parameter} must stay a string"
        );
    }
    assert_eq!(properties["priority"]["default"], "normal");

    // Escalation is always possible: the model may call the tool with no
    // arguments and let the attendant routing decide.
    assert_eq!(function["parameters"]["required"], serde_json::json!([]));
}
