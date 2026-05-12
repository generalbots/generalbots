pub mod mcp_add;
pub mod mcp_catalog;

use crate::state::AppState;
use axum::{extract::State, response::Html, routing::get, Router};
use std::sync::Arc;

pub async fn handle_sources_list_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Sources</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 1400px; margin: 0 auto; padding: 24px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.header h1 { font-size: 28px; color: #1a1a1a; }
.tabs { display: flex; gap: 8px; margin-bottom: 24px; border-bottom: 1px solid #ddd; padding-bottom: 16px; }
.tab { padding: 10px 20px; border: none; background: transparent; cursor: pointer; font-size: 14px; font-weight: 500; color: #666; border-radius: 8px; }
.tab.active { background: #0066cc; color: white; }
.tab:hover:not(.active) { background: #e8e8e8; }
.btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-primary:hover { background: #0052a3; }
.source-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 20px; }
.source-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.source-card:hover { box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
.source-header { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.source-icon { width: 40px; height: 40px; border-radius: 8px; background: #e8f4ff; display: flex; align-items: center; justify-content: center; font-size: 20px; }
.source-name { font-size: 16px; font-weight: 600; color: #1a1a1a; }
.source-type { font-size: 12px; color: #666; background: #f0f0f0; padding: 2px 8px; border-radius: 4px; }
.source-description { font-size: 14px; color: #666; margin-bottom: 12px; line-height: 1.5; }
.source-meta { display: flex; justify-content: space-between; align-items: center; }
.source-status { font-size: 12px; padding: 4px 8px; border-radius: 4px; }
.status-active { background: #e6f4ea; color: #1e7e34; }
.status-inactive { background: #fce8e6; color: #c5221f; }
.source-actions { display: flex; gap: 8px; }
.btn-sm { padding: 6px 12px; font-size: 12px; }
.btn-outline { background: transparent; border: 1px solid #ddd; color: #666; }
.btn-outline:hover { background: #f5f5f5; }
.empty-state { text-align: center; padding: 80px 24px; color: #666; }
.empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
.search-box { padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; width: 300px; }
.filters { display: flex; gap: 12px; margin-bottom: 24px; }
.filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
</style>
</head>
<body>
<div class="container">
<div class="header">
<h1>Sources</h1>
<button class="btn btn-primary" onclick="addSource()">Add Source</button>
</div>
<div class="tabs">
<button class="tab active" data-tab="mcp">MCP Servers</button>
<button class="tab" data-tab="repos">Repositories</button>
<button class="tab" data-tab="apps">Apps</button>
</div>
<div class="filters">
<input type="text" class="search-box" placeholder="Search sources..." id="searchInput">
<select class="filter-select" id="statusFilter">
<option value="">All Status</option>
<option value="active">Active</option>
<option value="inactive">Inactive</option>
</select>
</div>
<div class="source-grid" id="sourceGrid">
<div class="empty-state">
<h3>Loading sources...</h3>
</div>
</div>
</div>
<script>
let currentTab = 'mcp';

document.querySelectorAll('.tab').forEach(tab => {
tab.addEventListener('click', () => {
document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
tab.classList.add('active');
currentTab = tab.dataset.tab;
loadSources();
});
});

async function loadSources() {
const grid = document.getElementById('sourceGrid');
grid.innerHTML = '<div class="empty-state"><h3>Loading...</h3></div>';
try {
if (currentTab === 'mcp') { await loadMcpServers(); }
else if (currentTab === 'repos') { await loadRepositories(); }
else if (currentTab === 'apps') { await loadApps(); }
} catch (e) {
console.error('Failed to load sources:', e);
grid.innerHTML = '<div class="empty-state"><h3>Failed to load sources</h3></div>';
}
}

async function loadMcpServers() {
const response = await fetch('/api/ui/sources/mcp');
const data = await response.json();
const servers = data.data || data.servers || data || [];
renderMcpServers(Array.isArray(servers) ? servers : []);
}

async function loadRepositories() {
const response = await fetch('/api/ui/sources/repositories');
const data = await response.json();
const repos = data.data || data.repositories || data || [];
renderRepositories(Array.isArray(repos) ? repos : []);
}

async function loadApps() {
const response = await fetch('/api/ui/sources/apps');
const data = await response.json();
const apps = data.data || data.apps || data || [];
renderApps(Array.isArray(apps) ? apps : []);
}

function renderMcpServers(servers) {
const grid = document.getElementById('sourceGrid');
if (!servers || servers.length === 0) {
grid.innerHTML = '<div class="empty-state"><h3>No MCP servers configured</h3><p>Add an MCP server to extend your AI capabilities</p></div>';
return;
}
grid.innerHTML = servers.map(s => `
<div class="source-card">
<div class="source-header">
<div class="source-icon">🔌</div>
<div>
<div class="source-name">${escapeHtml(s.name)}</div>
<span class="source-type">${s.server_type || 'stdio'}</span>
</div>
</div>
<div class="source-description">${escapeHtml(s.description || 'No description')}</div>
<div class="source-meta">
<span class="source-status ${s.enabled ? 'status-active' : 'status-inactive'}">${s.enabled ? 'Active' : 'Inactive'}</span>
<span style="color: #666; font-size: 13px;">${s.tools_count || 0} tools</span>
</div>
<div class="source-actions" style="margin-top: 12px;">
<button class="btn btn-sm btn-outline" onclick="testServer('${escapeHtml(s.name)}')">Test</button>
<button class="btn btn-sm btn-outline" onclick="toggleServer('${escapeHtml(s.name)}', ${!s.enabled})">${s.enabled ? 'Disable' : 'Enable'}</button>
</div>
</div>
`).join('');
}

function renderRepositories(repos) {
const grid = document.getElementById('sourceGrid');
if (!repos || repos.length === 0) {
grid.innerHTML = '<div class="empty-state"><h3>No repositories connected</h3><p>Connect a repository to index your code</p></div>';
return;
}
grid.innerHTML = repos.map(r => `
<div class="source-card">
<div class="source-header">
<div class="source-icon">📁</div>
<div>
<div class="source-name">${escapeHtml(r.name)}</div>
<span class="source-type">${r.language || 'Unknown'}</span>
</div>
</div>
<div class="source-description">${escapeHtml(r.description || 'No description')}</div>
<div class="source-meta">
<span class="source-status ${r.status === 'synced' ? 'status-active' : 'status-inactive'}">${r.status || 'Unknown'}</span>
<span style="color: #666; font-size: 13px;">⭐ ${r.stars || 0}</span>
</div>
</div>
`).join('');
}

function renderApps(apps) {
const grid = document.getElementById('sourceGrid');
if (!apps || apps.length === 0) {
grid.innerHTML = '<div class="empty-state"><h3>No apps connected</h3><p>Connect apps to extend your workspace</p></div>';
return;
}
grid.innerHTML = apps.map(a => `
<div class="source-card">
<div class="source-header">
<div class="source-icon">📱</div>
<div>
<div class="source-name">${escapeHtml(a.name)}</div>
<span class="source-type">${a.app_type || 'app'}</span>
</div>
</div>
<div class="source-description">${escapeHtml(a.description || 'No description')}</div>
<div class="source-meta">
<span class="source-status ${a.status === 'active' ? 'status-active' : 'status-inactive'}">${a.status || 'Unknown'}</span>
</div>
</div>
`).join('');
}

async function testServer(name) {
try {
const response = await fetch('/api/ui/sources/mcp/' + encodeURIComponent(name) + '/test', { method: 'POST' });
const data = await response.json();
alert(data.success ? 'Server is working!' : 'Server test failed');
} catch (e) { alert('Failed to test server: ' + e.message); }
}

async function toggleServer(name, enable) {
try {
const endpoint = enable ? 'enable' : 'disable';
await fetch('/api/ui/sources/mcp/' + encodeURIComponent(name) + '/' + endpoint, { method: 'POST' });
loadSources();
} catch (e) { alert('Failed to toggle server: ' + e.message); }
}

function addSource() {
if (currentTab === 'mcp') { window.location = '/suite/sources/mcp/add'; }
else if (currentTab === 'repos') { window.location = '/suite/sources/repos/connect'; }
else { alert('Coming soon!'); }
}

function escapeHtml(str) {
if (!str) return '';
return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

loadSources();
</script>
</body>
</html>"#;
    Html(html.to_string())
}

pub fn configure_sources_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/sources", get(handle_sources_list_page))
        .route("/suite/sources/mcp/add", get(mcp_add::handle_mcp_add_page))
        .route(
            "/suite/sources/mcp/catalog",
            get(mcp_catalog::handle_mcp_catalog_page),
        )
}
