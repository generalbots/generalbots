use axum::extract::Path;
use axum::Json;
use botvibe::knowledge_graph::{get_knowledge_graph, get_run_graph};

#[tokio::test]
async fn use_case_graph_has_root_node() {
    let Json(response) = get_knowledge_graph(Path("software_development".to_string())).await;
    assert!(response.success);
    let graph = response.graph.expect("graph present on success");
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes[0].id, "root");
    assert_eq!(graph.nodes[0].node_type, "use_case");
    assert!(graph.nodes[0].label.contains("software_development"));
    assert!(graph.edges.is_empty());
    assert!(response.error.is_none());
}

#[tokio::test]
async fn run_graph_has_run_node() {
    let Json(response) = get_run_graph(Path("run-42".to_string())).await;
    assert!(response.success);
    let graph = response.graph.expect("graph present on success");
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes[0].id, "run");
    assert_eq!(graph.nodes[0].node_type, "run");
    assert!(graph.nodes[0].label.contains("run-42"));
    assert!(graph.edges.is_empty());
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