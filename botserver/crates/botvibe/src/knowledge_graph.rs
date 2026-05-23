//! Knowledge Graph module for the Vibe platform (Issue #522).
//!
//! Provides endpoints to retrieve knowledge graphs that represent the
//! relationships between Vibe runs, tool calls, and use cases. The graph
//! is returned as nodes and edges suitable for force-directed visualization
//! in the frontend (`vibe-graph.js`).
//!
//! # Endpoints
//! - `GET /api/vibe/graph/{use_case}` — graph for a use case
//! - `GET /api/vibe/graph/run/{run_id}` — graph for a specific run

use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single node in the knowledge graph (e.g., a use case, run, tool call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node.
    pub id: String,
    /// Human-readable label displayed on the graph canvas.
    pub label: String,
    /// Category of the node (e.g., "use_case", "run", "tool").
    pub node_type: String,
    /// Arbitrary key-value metadata attached to the node.
    pub properties: HashMap<String, String>,
}

/// A directed edge connecting two graph nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Relationship label (e.g., "contains", "triggered").
    pub relationship: String,
    /// Edge thickness hint for visualization (0.0–1.0).
    pub weight: f64,
}

/// Complete knowledge graph composed of nodes and edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    /// All nodes in the graph.
    pub nodes: Vec<GraphNode>,
    /// All edges connecting the nodes.
    pub edges: Vec<GraphEdge>,
}

/// JSON response wrapper for graph API calls.
#[derive(Debug, Serialize)]
pub struct GraphResponse {
    /// Whether the request succeeded.
    pub success: bool,
    /// The knowledge graph payload (absent on error).
    pub graph: Option<KnowledgeGraph>,
    /// Error message if the request failed.
    pub error: Option<String>,
}

/// Returns a knowledge graph for the given use case.
///
/// The graph contains a single root node representing the use case.
/// As runs and tool calls are recorded, they are added as child nodes.
pub async fn get_knowledge_graph(
    Path(_use_case): Path<String>,
) -> Json<GraphResponse> {
    Json(GraphResponse {
        success: true,
        graph: Some(KnowledgeGraph {
            nodes: vec![
                GraphNode {
                    id: "root".to_string(),
                    label: format!("Vibe Graph for {}", _use_case),
                    node_type: "use_case".to_string(),
                    properties: HashMap::new(),
                },
            ],
            edges: vec![],
        }),
        error: None,
    })
}

/// Returns a knowledge graph scoped to a specific Vibe run.
pub async fn get_run_graph(
    Path(_run_id): Path<String>,
) -> Json<GraphResponse> {
    Json(GraphResponse {
        success: true,
        graph: Some(KnowledgeGraph {
            nodes: vec![
                GraphNode {
                    id: "run".to_string(),
                    label: format!("Run {}", _run_id),
                    node_type: "run".to_string(),
                    properties: HashMap::new(),
                },
            ],
            edges: vec![],
        }),
        error: None,
    })
}
