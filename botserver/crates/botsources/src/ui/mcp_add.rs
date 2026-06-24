use crate::state::AppState;
use axum::{extract::State, response::Html};
use std::sync::Arc;

pub async fn handle_mcp_add_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Add MCP Server</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 800px; margin: 0 auto; padding: 24px; }
.back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
.card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
h1 { font-size: 24px; margin-bottom: 24px; }
.form-group { margin-bottom: 20px; }
.form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
.form-group textarea { min-height: 80px; resize: vertical; }
.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-primary:hover { background: #0052a3; }
.btn-outline { background: transparent; border: 1px solid #ddd; color: #666; }
.connection-section { margin-top: 16px; padding: 16px; background: #f8f8f8; border-radius: 8px; }
.connection-section h3 { font-size: 14px; margin-bottom: 12px; color: #666; }
.help-text { font-size: 12px; color: #888; margin-top: 4px; }
</style>
</head>
<body>
<div class="container">
<a href="/suite/sources" class="back-link">← Back to Sources</a>
<div class="card">
<h1>Add MCP Server</h1>
<form id="addServerForm">
<div class="form-row">
<div class="form-group">
<label>Name</label>
<input type="text" id="name" required placeholder="e.g., GitHub MCP">
</div>
<div class="form-group">
<label>Server Type</label>
<select id="serverType" onchange="updateConnectionFields()">
<option value="stdio">Stdio (Local Command)</option>
<option value="http">HTTP</option>
<option value="websocket">WebSocket</option>
</select>
</div>
</div>
<div class="form-group">
<label>Description</label>
<textarea id="description" placeholder="Describe what this MCP server does"></textarea>
</div>

<div class="connection-section" id="stdioSection">
<h3>Stdio Connection</h3>
<div class="form-group">
<label>Command</label>
<input type="text" id="command" placeholder="e.g., npx">
<div class="help-text">The command to run the MCP server</div>
</div>
<div class="form-group">
<label>Arguments (comma separated)</label>
<input type="text" id="args" placeholder="e.g., -y, @modelcontextprotocol/server-github">
</div>
</div>

<div class="connection-section" id="httpSection" style="display: none;">
<h3>HTTP Connection</h3>
<div class="form-group">
<label>URL</label>
<input type="text" id="httpUrl" placeholder="https://example.com/mcp">
</div>
<div class="form-group">
<label>Timeout (ms)</label>
<input type="number" id="timeout" value="30000">
</div>
</div>

<div class="connection-section" id="wsSection" style="display: none;">
<h3>WebSocket Connection</h3>
<div class="form-group">
<label>WebSocket URL</label>
<input type="text" id="wsUrl" placeholder="wss://example.com/mcp">
</div>
</div>

<div class="form-group">
<label>Tags (comma separated)</label>
<input type="text" id="tags" placeholder="e.g., github, code, productivity">
</div>

<div class="form-group">
<label>
<input type="checkbox" id="enabled" checked> Enable immediately
</label>
</div>
<div class="form-group">
<label>
<input type="checkbox" id="requiresApproval"> Require approval for tool calls
</label>
</div>

<div style="display: flex; gap: 12px; margin-top: 24px;">
<button type="submit" class="btn btn-primary">Add Server</button>
<button type="button" class="btn btn-outline" onclick="window.location='/suite/sources'">Cancel</button>
</div>
</form>
</div>
</div>
<script>
function updateConnectionFields() {
const serverType = document.getElementById('serverType').value;
document.getElementById('stdioSection').style.display = serverType === 'stdio' ? 'block' : 'none';
document.getElementById('httpSection').style.display = serverType === 'http' ? 'block' : 'none';
document.getElementById('wsSection').style.display = serverType === 'websocket' ? 'block' : 'none';
}

document.getElementById('addServerForm').addEventListener('submit', async (e) => {
e.preventDefault();

const serverType = document.getElementById('serverType').value;
let connection = {};

if (serverType === 'stdio') {
const args = document.getElementById('args').value.split(',').map(a => a.trim()).filter(a => a);
connection = { Stdio: { command: document.getElementById('command').value, args } };
} else if (serverType === 'http') {
connection = { Http: { url: document.getElementById('httpUrl').value, timeout: parseInt(document.getElementById('timeout').value) } };
} else if (serverType === 'websocket') {
connection = { WebSocket: { url: document.getElementById('wsUrl').value } };
}

const tags = document.getElementById('tags').value.split(',').map(t => t.trim()).filter(t => t);

const payload = {
name: document.getElementById('name').value,
description: document.getElementById('description').value,
server_type: serverType,
connection,
auth: { None: null },
enabled: document.getElementById('enabled').checked,
tags,
requires_approval: document.getElementById('requiresApproval').checked
};

try {
const response = await fetch('/api/ui/sources/mcp', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify(payload)
});

if (response.ok) { window.location = '/suite/sources'; }
else {
const data = await response.json();
alert('Failed to add server: ' + (data.error || 'Unknown error'));
}
} catch (e) { alert('Failed to add server: ' + e.message); }
});
</script>
</body>
</html>"#;
    Html(html.to_string())
}
