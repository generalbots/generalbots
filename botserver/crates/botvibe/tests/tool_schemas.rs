use botvibe::domains_tool::{domain_bind_schema, domain_tls_schema, domain_verify_schema};
use botvibe::ops_tools::ops_tools;
use botvibe::publish::publish_project_schema;
use botvibe::types::VibeUseCase;

#[test]
fn publish_schema_requires_project_id_and_approval() {
    let schema = publish_project_schema();
    assert_eq!(schema.name, "publish/project");
    assert!(schema.requires_approval);
    assert_eq!(schema.allowed_use_cases, vec![VibeUseCase::SoftwareDevelopment]);
    let required = schema.parameters["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "project_id");
    let env_enum = schema.parameters["properties"]["env"]["enum"].as_array().unwrap();
    assert!(env_enum.iter().any(|v| v == "production"));
}

#[test]
fn domain_schemas_require_domain_and_approval() {
    let bind = domain_bind_schema();
    assert_eq!(bind.name, "domain/bind");
    assert!(bind.requires_approval);
    let required: Vec<&str> = bind.parameters["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, vec!["project_id", "domain"]);

    let verify = domain_verify_schema();
    assert_eq!(verify.name, "domain/verify");
    assert!(verify.requires_approval);

    let tls = domain_tls_schema();
    assert_eq!(tls.name, "domain/tls");
    assert!(tls.requires_approval);
}

#[test]
fn ops_tools_registry_shapes() {
    let tools = ops_tools();
    let names: Vec<&str> = tools.iter().map(|(name, _, _)| *name).collect();
    assert_eq!(
        names,
        vec![
            "vm/probe",
            "vm/restart",
            "publish/history",
            "publish/rollback",
            "backup/snapshot",
            "backup/export",
            "backup/list",
            "backup/restore",
        ]
    );

    for (name, schema, _) in tools {
        assert_eq!(schema.name, name);
        assert_eq!(schema.allowed_use_cases, vec![VibeUseCase::SoftwareDevelopment]);
        let required = schema.parameters["required"].as_array().unwrap();
        assert!(!required.is_empty(), "{name} must declare required params");
        let destructive = ["vm/restart", "publish/rollback", "backup/snapshot", "backup/export", "backup/restore"];
        let read_only = ["vm/probe", "publish/history", "backup/list"];
        if destructive.contains(&name) {
            assert!(schema.requires_approval, "{name} should require approval");
        } else if read_only.contains(&name) {
            assert!(!schema.requires_approval, "{name} should not require approval");
        }
    }
}