//! Manifest guard (#1678 follow-up): generated `.mcp.json` manifests must
//! carry MCP annotations when the tool source declares them, must validate
//! against the tool schema, and unknown annotation hints must not break
//! compilation of older scripts.

use botbasic_compiler::BasicCompiler;

fn compile_manifest(source: &str) -> serde_json::Value {
    let manifest = BasicCompiler::manifest_json(source, "deploy.bas")
        .expect("manifest compiles");
    serde_json::from_str(&manifest).expect("manifest is valid JSON")
}

#[test]
fn annotations_flow_from_bas_to_manifest() {
    let manifest = compile_manifest(
        "DESCRIPTION \"Deploys a build to a target environment.\"\n\
         PARAM url AS string DESCRIPTION \"Deploy target URL\"\n\
         ANNOTATION READ_ONLY\n\
         ANNOTATION IDEMPOTENT\n\
         ANNOTATION TITLE \"Deploy\"\n",
    );
    let annotations = manifest.get("annotations").expect("annotations recorded");
    assert_eq!(annotations.get("title"), Some(&serde_json::json!("Deploy")));
    assert_eq!(
        annotations.get("readOnlyHint"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        annotations.get("idempotentHint"),
        Some(&serde_json::json!(true))
    );
    // Schema still present and well-formed next to the hints.
    assert_eq!(
        manifest.pointer("/input_schema/type"),
        Some(&serde_json::json!("object"))
    );
    assert_eq!(
        manifest.pointer("/input_schema/properties/url/type"),
        Some(&serde_json::json!("string"))
    );
}

#[test]
fn unknown_annotation_hints_are_ignored() {
    let manifest = compile_manifest(
        "DESCRIPTION \"Tool with a hint a future compiler might add.\"\n\
         ANNOTATION SOME_FUTURE_HINT\n",
    );
    assert!(
        manifest.get("annotations").is_none(),
        "an unknown hint must not produce a half-filled annotations object"
    );
}

#[test]
fn tools_without_annotations_keep_the_plain_manifest() {
    let manifest = compile_manifest(
        "DESCRIPTION \"Plain tool.\"\n\
         PARAM name AS string DESCRIPTION \"Argument name\"\n",
    );
    assert!(manifest.get("annotations").is_none());
    assert_eq!(manifest.get("name"), Some(&serde_json::json!("deploy")));
    assert!(manifest.get("input_schema").is_some());
}
