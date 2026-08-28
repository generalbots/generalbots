use botvibe::knowledge_graph::{
    build_run_graph, build_use_case_graph, GraphDataSource, GraphFuture, RunNodeInfo,
};

struct MockGraphSource {
    runs: Vec<RunNodeInfo>,
}

impl GraphDataSource for MockGraphSource {
    fn snapshot_runs(&self) -> GraphFuture<Vec<RunNodeInfo>> {
        let runs = self.runs.clone();
        Box::pin(async move { runs })
    }
}

fn sw_dev_run(id: &str, tools: &[&str]) -> RunNodeInfo {
    RunNodeInfo {
        run_id: id.to_string(),
        use_case: "software_development".to_string(),
        state: "completed".to_string(),
        intent: "Add login page".to_string(),
        tool_names: tools.iter().map(|t| t.to_string()).collect(),
        project_id: Some("00000000-0000-0000-0000-0000000000aa".to_string()),
    }
}

#[test]
fn use_case_graph_has_root_run_and_tool_nodes() {
    let runs = vec![
        sw_dev_run("00000000-0000-0000-0000-000000000001", &["write_file", "run_tests"]),
        sw_dev_run("00000000-0000-0000-0000-000000000002", &["run_tests"]),
        RunNodeInfo {
            run_id: "cs-run".to_string(),
            use_case: "customer_support".to_string(),
            state: "running".to_string(),
            intent: "Help with ticket".to_string(),
            tool_names: vec!["search_kb".to_string()],
            project_id: None,
        },
    ];

    let graph = build_use_case_graph("software_development", &runs, None);
    let kinds = graph
        .nodes
        .iter()
        .map(|n| n.node_type.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds.iter().filter(|k| **k == "use_case").count(),
        1,
        "exactly one root use_case node"
    );
    assert_eq!(
        kinds.iter().filter(|k| **k == "run").count(),
        2,
        "customer_support run must be filtered out"
    );
    assert_eq!(
        kinds.iter().filter(|k| **k == "tool").count(),
        2,
        "tools are deduplicated by name"
    );

    assert!(graph
        .edges
        .iter()
        .any(|e| e.relationship == "contains" && e.source == "root"));
    assert!(graph
        .edges
        .iter()
        .any(|e| e.relationship == "triggered" && e.target == "tool:run_tests"));

    let tool_node = graph
        .nodes
        .iter()
        .find(|n| n.id == "tool:run_tests")
        .expect("deduplicated tool node exists");
    assert_eq!(tool_node.properties["calls"], "2");
}

#[test]
fn use_case_graph_scopes_to_project_when_requested() {
    let runs = vec![
        sw_dev_run("00000000-0000-0000-0000-000000000001", &["write_file"]),
        RunNodeInfo {
            run_id: "00000000-0000-0000-0000-000000000002".to_string(),
            use_case: "software_development".to_string(),
            state: "completed".to_string(),
            intent: "Other project".to_string(),
            tool_names: vec!["deploy_app".to_string()],
            project_id: Some("00000000-0000-0000-0000-0000000000bb".to_string()),
        },
    ];
    let graph = build_use_case_graph(
        "software_development",
        &runs,
        Some("00000000-0000-0000-0000-0000000000aa"),
    );
    assert_eq!(
        graph
            .nodes
            .iter()
            .filter(|n| n.node_type == "run")
            .count(),
        1,
        "only the run belonging to the requested project appears"
    );
    assert!(graph.nodes.iter().all(|n| n.id != "tool:deploy_app"));
}

#[test]
fn use_case_graph_with_only_foreign_runs_keeps_root() {
    let runs = vec![RunNodeInfo {
        run_id: "cs-run".to_string(),
        use_case: "customer_support".to_string(),
        state: "running".to_string(),
        intent: "Help".to_string(),
        tool_names: vec![],
        project_id: None,
    }];
    let graph = build_use_case_graph("financial_analysis", &runs, None);
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes[0].id, "root");
    assert!(graph.edges.is_empty());
}

#[test]
fn run_graph_scopes_to_single_run_and_tools() {
    let runs = vec![
        sw_dev_run("00000000-0000-0000-0000-000000000001", &["write_file", "run_tests"]),
        sw_dev_run("00000000-0000-0000-0000-000000000002", &["deploy_app"]),
    ];

    let graph = build_run_graph("00000000-0000-0000-0000-000000000001", &runs)
        .expect("run exists in snapshot");
    assert_eq!(graph.nodes.len(), 3, "run node plus two tool nodes");
    assert_eq!(graph.nodes[0].node_type, "run");
    assert_eq!(graph.nodes[0].properties["intent"], "Add login page");
    assert_eq!(graph.edges.len(), 2);
    assert!(graph
        .edges
        .iter()
        .all(|e| e.source == "run:00000000-0000-0000-0000-000000000001"));

    let missing = build_run_graph("00000000-0000-0000-0000-000000000009", &runs);
    assert!(missing.is_err());
    assert!(missing.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn builders_consume_snapshot_from_data_source() {
    let source = MockGraphSource {
        runs: vec![sw_dev_run("00000000-0000-0000-0000-000000000001", &["write_file"])],
    };
    let runs = source.snapshot_runs().await;
    let graph = build_use_case_graph("software_development", &runs, None);
    assert_eq!(graph.nodes.len(), 3);
}

#[test]
fn graph_types_serialize_round_trip() {
    use botvibe::knowledge_graph::{GraphEdge, GraphNode, KnowledgeGraph};
    use std::collections::HashMap;

    let graph = KnowledgeGraph {
        nodes: vec![GraphNode {
            id: "n1".to_string(),
            label: "Node 1".to_string(),
            node_type: "tool".to_string(),
            properties: HashMap::from([("status".to_string(), "ok".to_string())]),
        }],
        edges: vec![GraphEdge {
            source: "root".to_string(),
            target: "n1".to_string(),
            relationship: "contains".to_string(),
            weight: 0.5,
        }],
    };
    let value = serde_json::to_value(&graph).unwrap();
    assert_eq!(value["nodes"][0]["node_type"], "tool");
    assert_eq!(value["nodes"][0]["properties"]["status"], "ok");
    assert_eq!(value["edges"][0]["relationship"], "contains");
    assert_eq!(value["edges"][0]["weight"], 0.5);

    let decoded: KnowledgeGraph = serde_json::from_value(value).unwrap();
    assert_eq!(decoded.nodes.len(), 1);
    assert_eq!(decoded.edges[0].target, "n1");
}