use crate::state::AppState;
use axum::{extract::State, response::Html};
use std::sync::Arc;

pub async fn handle_mcp_catalog_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>MCP Server Catalog</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
.container { max-width: 1400px; margin: 0 auto; padding: 24px; }
.back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
.header h1 { font-size: 28px; color: #1a1a1a; }
.filters { display: flex; gap: 12px; margin-bottom: 24px; }
.search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; max-width: 400px; }
.filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
.catalog-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; }
.catalog-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; }
.catalog-card:hover { box-shadow: 0 4px 16px rgba(0,0,0,0.12); transform: translateY(-2px); transition: all 0.2s; }
.catalog-header { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.catalog-icon { width: 48px; height: 48px; border-radius: 10px; background: #e8f4ff; display: flex; align-items: center; justify-content: center; font-size: 24px; }
.catalog-name { font-size: 16px; font-weight: 600; color: #1a1a1a; }
.catalog-provider { font-size: 12px; color: #888; }
.catalog-description { font-size: 14px; color: #666; margin-bottom: 12px; line-height: 1.5; }
.catalog-category { font-size: 12px; color: #666; background: #f0f0f0; padding: 4px 8px; border-radius: 4px; display: inline-block; }
.btn { padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 13px; font-weight: 500; }
.btn-primary { background: #0066cc; color: white; }
.btn-primary:hover { background: #0052a3; }
</style>
</head>
<body>
<div class="container">
<a href="/suite/sources" class="back-link">← Back to Sources</a>
<div class="header">
<h1>MCP Server Catalog</h1>
</div>
<div class="filters">
<input type="text" class="search-box" placeholder="Search servers..." id="searchInput" oninput="filterCatalog()">
<select class="filter-select" id="categoryFilter" onchange="filterCatalog()">
<option value="">All Categories</option>
</select>
</div>
<div class="catalog-grid" id="catalogGrid">
<div style="text-align: center; padding: 40px; color: #666;">Loading catalog...</div>
</div>
</div>
<script>
let allServers = [];
let categories = [];

async function loadCatalog() {
try {
const response = await fetch('/api/ui/sources/mcp-servers');
const data = await response.json();
allServers = data.mcp_servers || [];
categories = data.categories || [];

const categorySelect = document.getElementById('categoryFilter');
categories.forEach(cat => {
const option = document.createElement('option');
option.value = cat;
option.textContent = cat;
categorySelect.appendChild(option);
});

renderCatalog(allServers);
} catch (e) {
console.error('Failed to load catalog:', e);
document.getElementById('catalogGrid').innerHTML = '<div style="text-align: center; padding: 40px; color: #666;">Failed to load catalog</div>';
}
}

function filterCatalog() {
const search = document.getElementById('searchInput').value.toLowerCase();
const category = document.getElementById('categoryFilter').value;

let filtered = allServers;
if (search) {
filtered = filtered.filter(s =>
s.name.toLowerCase().includes(search) ||
(s.description && s.description.toLowerCase().includes(search))
);
}
if (category) {
filtered = filtered.filter(s => s.category === category);
}
renderCatalog(filtered);
}

function renderCatalog(servers) {
const grid = document.getElementById('catalogGrid');
if (!servers || servers.length === 0) {
grid.innerHTML = '<div style="text-align: center; padding: 40px; color: #666;">No servers found</div>';
return;
}
grid.innerHTML = servers.map(s => `
<div class="catalog-card" onclick="installServer('${s.id}')">
<div class="catalog-header">
<div class="catalog-icon">${s.icon || '🔌'}</div>
<div>
<div class="catalog-name">${escapeHtml(s.name)}</div>
<div class="catalog-provider">${escapeHtml(s.provider || 'Community')}</div>
</div>
</div>
<div class="catalog-description">${escapeHtml(s.description || 'No description')}</div>
<div style="display: flex; justify-content: space-between; align-items: center;">
<span class="catalog-category">${escapeHtml(s.category || 'General')}</span>
<button class="btn btn-primary">Install</button>
</div>
</div>
`).join('');
}

async function installServer(id) {
if (confirm('Install this MCP server?')) {
alert('Server installation initiated. Check the Sources page for status.');
window.location = '/suite/sources';
}
}

function escapeHtml(str) {
if (!str) return '';
return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

loadCatalog();
</script>
</body>
</html>"#;
    Html(html.to_string())
}
