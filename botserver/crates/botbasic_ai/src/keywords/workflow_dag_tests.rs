// Workflow DAG keyword tests split from `workflow_dag.rs` to keep the
// implementation file under the AGENTS.md 450-line limit.

#![cfg(test)]

use super::*;

#[test]
fn node_kind_serde_round_trip() {
    let serialized = serde_json::to_string(&NodeKind::Branch).unwrap();
    assert_eq!(serialized, "\"branch\"");
    let deserialized: NodeKind = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, NodeKind::Branch);
}

#[test]
fn dag_state_default_is_step_running() {
    let dag = DagState::default();
    assert_eq!(dag.status, "");
    assert_eq!(dag.next_step, 0);
    assert!(dag.nodes.is_empty());
}

#[test]
fn dag_node_includes_branches() {
    let node = DagNode {
        id: Uuid::new_v4(),
        kind: NodeKind::Parallel,
        label: "fanout".into(),
        condition: None,
        branches: vec!["a".into(), "b".into()],
        handler: None,
        depends_on: vec![],
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("\"branches\":[\"a\",\"b\"]"));
}
