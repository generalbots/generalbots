use crate::core::shared::state::AppState;
use crate::designer::bas_analyzer::{BasFileAnalyzer, BasFileType, WorkflowMetadata};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: NodeType,
    pub position: Position,
    pub config: NodeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NodeType {
    BotAgent { bot_name: String, action: String },
    HumanApproval { approver: String, timeout: u32 },
    Condition { expression: String },
    Parallel { branches: Vec<String> },
    Event { event_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub label: String,
    pub description: Option<String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConnection {
    pub from_node: String,
    pub to_node: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCanvas {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<WorkflowNode>,
    pub connections: Vec<WorkflowConnection>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl WorkflowCanvas {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            nodes: Vec::new(),
            connections: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn generate_basic_code(&self) -> String {
        let mut code = format!("' Generated workflow: {}\n", self.name);
        code.push_str(&format!("ORCHESTRATE WORKFLOW \"{}\"\n", self.name));
        
        let mut step_counter = 1;
        for node in &self.nodes {
            match &node.node_type {
                NodeType::BotAgent { bot_name, action } => {
                    code.push_str(&format!("  STEP {}: BOT \"{}\" \"{}\"\n", step_counter, bot_name, action));
                    step_counter += 1;
                }
                NodeType::HumanApproval { approver, timeout } => {
                    code.push_str(&format!("  STEP {}: HUMAN APPROVAL FROM \"{}\"\n", step_counter, approver));
                    code.push_str(&format!("    TIMEOUT {}\n", timeout));
                    step_counter += 1;
                }
                NodeType::Condition { expression } => {
                    code.push_str(&format!("  IF {} THEN\n", expression));
                }
                NodeType::Parallel { branches: _ } => {
                    code.push_str(&format!("  STEP {}: PARALLEL\n", step_counter));
                    code.push_str("    BRANCH A: BOT \"branch-a\" \"process\"\n");
                    code.push_str("    BRANCH B: BOT \"branch-b\" \"process\"\n");
                    code.push_str("  END PARALLEL\n");
                    step_counter += 1;
                }
                NodeType::Event { event_name } => {
                    code.push_str(&format!("  PUBLISH EVENT \"{}\"\n", event_name));
                }
            }
        }
        
        code.push_str("END WORKFLOW\n");
        code
    }
    
    pub fn detect_file_type(content: &str) -> BasFileType {
        match BasFileAnalyzer::analyze_content(content) {
            Ok(file_type) => file_type,
            Err(_) => BasFileType::Regular,
        }
    }
    
    pub fn get_metadata(&self) -> WorkflowMetadata {
        let code = self.generate_basic_code();
        BasFileAnalyzer::get_workflow_metadata(&code)
    }
}

pub async fn workflow_designer_page(
    State(_state): State<Arc<AppState>>,
) -> Result<Html<String>, StatusCode> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Workflow Designer</title>
    <script src="/static/htmx.min.js"></script>
    <style>
        .canvas { 
            width: 100%; 
            height: 600px; 
            border: 1px solid #ccc; 
            position: relative;
            background: #f9f9f9;
        }
        .node { 
            position: absolute; 
            padding: 10px; 
            border: 2px solid #333; 
            background: white;
            border-radius: 5px;
            cursor: move;
            min-width: 120px;
            text-align: center;
        }
        .node.bot-agent { border-color: #007bff; background: #e7f3ff; }
        .node.human-approval { border-color: #ffc107; background: #fff8e1; }
        .node.condition { border-color: #28a745; background: #e8f5e9; }
        .node.parallel { border-color: #6f42c1; background: #f3e5f5; }
        .node.event { border-color: #fd7e14; background: #fff3e0; }
        .toolbar { 
            padding: 10px; 
            background: #f8f9fa; 
            border-bottom: 1px solid #dee2e6;
        }
        .btn { 
            padding: 8px 16px; 
            margin: 0 5px; 
            border: none; 
            border-radius: 4px; 
            cursor: pointer;
        }
        .btn-primary { background: #007bff; color: white; }
        .btn-success { background: #28a745; color: white; }
        .btn-warning { background: #ffc107; color: black; }
        .code-preview { 
            margin-top: 20px; 
            padding: 15px; 
            background: #f8f9fa; 
            border: 1px solid #dee2e6;
            font-family: monospace;
            white-space: pre-wrap;
        }
    </style>
</head>
<body>
    <div class="toolbar">
        <h2>Workflow Designer</h2>
        <button class="btn btn-primary" onclick="addNode('bot-agent')">Add Bot</button>
        <button class="btn btn-warning" onclick="addNode('human-approval')">Add Approval</button>
        <button class="btn btn-success" onclick="addNode('condition')">Add Condition</button>
        <button class="btn btn-primary" onclick="addNode('parallel')">Add Parallel</button>
        <button class="btn" onclick="addNode('event')">Add Event</button>
        <button class="btn btn-success" hx-post="/api/workflow/generate" hx-target="#code-preview">Generate Code</button>
        <input type="file" id="file-input" accept=".bas" onchange="analyzeFile()" style="margin-left: 20px;">
        <label for="file-input" class="btn">Analyze .bas File</label>
    </div>
    
    <div id="file-analysis" style="display:none; padding: 10px; background: #e8f4f8; border: 1px solid #bee5eb; margin: 10px 0;">
        <h4>File Analysis Result</h4>
        <div id="analysis-content"></div>
    </div>
    
    <div id="canvas" class="canvas" ondrop="drop(event)" ondragover="allowDrop(event)">
    </div>
    
    <div id="code-preview" class="code-preview">
        Generated BASIC code will appear here...
    </div>

    <script>
        let nodeCounter = 0;
        let nodes = [];
        
        function addNode(type) {
            nodeCounter++;
            const node = {
                id: 'node-' + nodeCounter,
                type: type,
                x: 50 + (nodeCounter * 20),
                y: 50 + (nodeCounter * 20)
            };
            nodes.push(node);
            renderNode(node);
        }
        
        function renderNode(node) {
            const canvas = document.getElementById('canvas');
            const nodeEl = document.createElement('div');
            nodeEl.className = 'node ' + node.type;
            nodeEl.id = node.id;
            nodeEl.draggable = true;
            nodeEl.style.left = node.x + 'px';
            nodeEl.style.top = node.y + 'px';
            
            let content = '';
            switch(node.type) {
                case 'bot-agent':
                    content = '<strong>Bot Agent</strong><br><input type="text" placeholder="Bot Name" style="width:100px;margin:2px;"><br><input type="text" placeholder="Action" style="width:100px;margin:2px;">';
                    break;
                case 'human-approval':
                    content = '<strong>Human Approval</strong><br><input type="text" placeholder="Approver" style="width:100px;margin:2px;"><br><input type="number" placeholder="Timeout" style="width:100px;margin:2px;">';
                    break;
                case 'condition':
                    content = '<strong>Condition</strong><br><input type="text" placeholder="Expression" style="width:100px;margin:2px;">';
                    break;
                case 'parallel':
                    content = '<strong>Parallel</strong><br>Multiple branches';
                    break;
                case 'event':
                    content = '<strong>Event</strong><br><input type="text" placeholder="Event Name " style="width:100px;margin:2px;">';
                    break;
            }
            
            nodeEl.innerHTML = content;
            nodeEl.ondragstart = drag;
            canvas.appendChild(nodeEl);
        }
        
        function allowDrop(ev) {
            ev.preventDefault();
        }
        
        function drag(ev) {
            ev.dataTransfer.setData("text", ev.target.id);
        }
        
        function drop(ev) {
            ev.preventDefault();
            const data = ev.dataTransfer.getData("text");
            const nodeEl = document.getElementById(data);
            const rect = ev.currentTarget.getBoundingClientRect();
            const x = ev.clientX - rect.left;
            const y = ev.clientY - rect.top;
            
            nodeEl.style.left = x + 'px';
            nodeEl.style.top = y + 'px';
            
            // Update node position in data
            const node = nodes.find(n => n.id === data);
            if (node) {
                node.x = x;
                node.y = y;
            }
        }
        
        function analyzeFile() {
            const fileInput = document.getElementById('file-input');
            const file = fileInput.files[0];
            
            if (file) {
                const reader = new FileReader();
                reader.onload = function(e) {
                    const content = e.target.result;
                    
                    fetch('/api/workflow/analyze', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({ content: content })
                    })
                    .then(response => response.json())
                    .then(data => {
                        displayAnalysis(data);
                    })
                    .catch(error => {
                        console.error('Analysis failed:', error);
                    });
                };
                reader.readAsText(file);
            }
        }
        
        function displayAnalysis(analysis) {
            const analysisDiv = document.getElementById('file-analysis');
            const contentDiv = document.getElementById('analysis-content');
            
            let html = `<p><strong>File Type:</strong> ${analysis.file_type}</p>`;
            
            if (analysis.metadata) {
                html += `<p><strong>Workflow Name:</strong> ${analysis.metadata.name}</p>`;
                html += `<p><strong>Steps:</strong> ${analysis.metadata.step_count}</p>`;
                html += `<p><strong>Bots Used:</strong> ${analysis.metadata.bot_count}</p>`;
                html += `<p><strong>Human Approval:</strong> ${analysis.metadata.has_human_approval ? 'Yes' : 'No'}</p>`;
                html += `<p><strong>Parallel Processing:</strong> ${analysis.metadata.has_parallel ? 'Yes' : 'No'}</p>`;
            }
            
            if (analysis.suggestions.length > 0) {
                html += '<p><strong>Suggestions:</strong></p><ul>';
                analysis.suggestions.forEach(suggestion => {
                    html += `<li>${suggestion}</li>`;
                });
                html += '</ul>';
            }
            
            contentDiv.innerHTML = html;
            analysisDiv.style.display = 'block';
        }
    </script>
</body>
</html>
    "#;
    
    Ok(Html(html.to_string()))
}

#[derive(Deserialize)]
pub struct GenerateCodeRequest {
    pub nodes: Vec<WorkflowNode>,
    pub connections: Vec<WorkflowConnection>,
}

pub async fn generate_workflow_code(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<GenerateCodeRequest>,
) -> Result<Html<String>, StatusCode> {
    let canvas = WorkflowCanvas {
        id: Uuid::new_v4(),
        name: "Generated Workflow".to_string(),
        nodes: request.nodes,
        connections: request.connections,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let code = canvas.generate_basic_code();
    Ok(Html(format!("<pre>{}</pre>", code)))
}

#[derive(Deserialize)]
pub struct AnalyzeFileRequest {
    pub content: String,
}

#[derive(Serialize)]
pub struct AnalyzeFileResponse {
    pub file_type: String,
    pub metadata: Option<WorkflowMetadata>,
    pub suggestions: Vec<String>,
}

pub async fn analyze_bas_file(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<AnalyzeFileRequest>,
) -> Result<Json<AnalyzeFileResponse>, StatusCode> {
    let file_type = WorkflowCanvas::detect_file_type(&request.content);
    
    let (type_str, metadata, suggestions) = match file_type {
        BasFileType::Workflow => {
            let meta = BasFileAnalyzer::get_workflow_metadata(&request.content);
            let mut suggestions = Vec::new();
            
            if meta.step_count > 10 {
                suggestions.push("Consider breaking this workflow into smaller sub-workflows".to_string());
            }
            if meta.bot_count > 5 {
                suggestions.push("High bot count - ensure proper resource management".to_string());
            }
            if !meta.has_human_approval && meta.step_count > 3 {
                suggestions.push("Consider adding human approval for complex workflows".to_string());
            }
            
            ("workflow".to_string(), Some(meta), suggestions)
        }
        BasFileType::Tool => {
            let suggestions = vec![
                "Tools should be simple and focused on single operations".to_string(),
                "Consider using USE TOOL instead of complex logic".to_string(),
            ];
            ("tool".to_string(), None, suggestions)
        }
        BasFileType::Regular => {
            let suggestions = vec![
                "Regular bot - consider upgrading to workflow for complex logic".to_string(),
            ];
            ("regular".to_string(), None, suggestions)
        }
    };
    
    Ok(Json(AnalyzeFileResponse {
        file_type: type_str,
        metadata,
        suggestions,
    }))
}
