use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

pub async fn handle_dashboards_list_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dashboards</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
        .header h1 { font-size: 28px; color: #1a1a1a; }
        .btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .tabs { display: flex; gap: 4px; margin-bottom: 24px; border-bottom: 1px solid #e0e0e0; }
        .tab { padding: 12px 24px; background: none; border: none; cursor: pointer; font-size: 14px; color: #666; border-bottom: 2px solid transparent; }
        .tab.active { color: #0066cc; border-bottom-color: #0066cc; }
        .dashboard-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 24px; }
        .dashboard-card { background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; }
        .dashboard-card:hover { transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
        .dashboard-preview { width: 100%; aspect-ratio: 16/9; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); display: flex; align-items: center; justify-content: center; color: white; font-size: 48px; }
        .dashboard-info { padding: 16px; }
        .dashboard-title { font-size: 16px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
        .dashboard-meta { font-size: 13px; color: #666; display: flex; gap: 12px; }
        .dashboard-tags { display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap; }
        .tag { padding: 4px 10px; background: #f0f0f0; border-radius: 4px; font-size: 12px; color: #666; }
        .filters { display: flex; gap: 12px; margin-bottom: 24px; }
        .search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; }
        .filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
        .empty-state { text-align: center; padding: 80px 24px; color: #666; }
        .empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
        .template-card { border: 2px dashed #ddd; background: #fafafa; }
        .template-card:hover { border-color: #0066cc; background: #f0f7ff; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Dashboards</h1>
            <button class="btn btn-primary" onclick="createDashboard()">+ New Dashboard</button>
        </div>
        <div class="tabs">
            <button class="tab active" data-view="my">My Dashboards</button>
            <button class="tab" data-view="shared">Shared with Me</button>
            <button class="tab" data-view="templates">Templates</button>
        </div>
        <div class="filters">
            <input type="text" class="search-box" placeholder="Search dashboards..." id="searchInput" oninput="filterDashboards()">
            <select class="filter-select" id="sortBy" onchange="filterDashboards()">
                <option value="updated">Recently Updated</option>
                <option value="created">Recently Created</option>
                <option value="name">Name A-Z</option>
            </select>
        </div>
        <div class="dashboard-grid" id="dashboardGrid">
            <div class="empty-state">
                <h3>No dashboards yet</h3>
                <p>Create your first dashboard to visualize your data</p>
            </div>
        </div>
    </div>
    <script>
        let dashboards = [];
        let currentView = 'my';

        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                currentView = tab.dataset.view;
                loadDashboards();
            });
        });

        async function loadDashboards() {
            try {
                const endpoint = currentView === 'templates' ? '/api/dashboards/templates' : '/api/dashboards';
                const response = await fetch(endpoint);
                dashboards = await response.json();
                renderDashboards();
            } catch (e) {
                console.error('Failed to load dashboards:', e);
            }
        }

        function renderDashboards() {
            const grid = document.getElementById('dashboardGrid');
            if (!dashboards || dashboards.length === 0) {
                grid.innerHTML = currentView === 'templates'
                    ? '<div class="empty-state"><h3>No templates available</h3><p>Templates will appear here when created</p></div>'
                    : '<div class="empty-state"><h3>No dashboards yet</h3><p>Create your first dashboard to visualize your data</p></div>';
                return;
            }

            grid.innerHTML = dashboards.map(d => `
                <div class="dashboard-card ${d.is_template ? 'template-card' : ''}" onclick="openDashboard('${d.id}')">
                    <div class="dashboard-preview">📊</div>
                    <div class="dashboard-info">
                        <div class="dashboard-title">${d.name}</div>
                        <div class="dashboard-meta">
                            <span>${d.widgets ? d.widgets.length : 0} widgets</span>
                            <span>Updated ${formatDate(d.updated_at)}</span>
                        </div>
                        ${d.tags && d.tags.length ? `<div class="dashboard-tags">${d.tags.map(t => `<span class="tag">${t}</span>`).join('')}</div>` : ''}
                    </div>
                </div>
            `).join('');
        }

        function formatDate(dateStr) {
            if (!dateStr) return 'Never';
            const date = new Date(dateStr);
            const now = new Date();
            const diff = now - date;
            if (diff < 86400000) return 'Today';
            if (diff < 172800000) return 'Yesterday';
            return date.toLocaleDateString();
        }

        function filterDashboards() {
            const query = document.getElementById('searchInput').value.toLowerCase();
            const filtered = dashboards.filter(d =>
                d.name.toLowerCase().includes(query) ||
                (d.description && d.description.toLowerCase().includes(query))
            );
            renderFilteredDashboards(filtered);
        }

        function renderFilteredDashboards(filtered) {
            const grid = document.getElementById('dashboardGrid');
            if (!filtered || filtered.length === 0) {
                grid.innerHTML = '<div class="empty-state"><h3>No dashboards found</h3><p>Try a different search term</p></div>';
                return;
            }
            dashboards = filtered;
            renderDashboards();
        }

        function createDashboard() {
            window.location = '/suite/dashboards/new';
        }

        function openDashboard(id) {
            window.location = `/suite/dashboards/${id}`;
        }

        loadDashboards();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_dashboard_detail_page(
    State(_state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dashboard</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
        .header {{ background: white; padding: 16px 24px; border-bottom: 1px solid #e0e0e0; display: flex; justify-content: space-between; align-items: center; }}
        .header-left {{ display: flex; align-items: center; gap: 16px; }}
        .back-link {{ color: #666; text-decoration: none; font-size: 20px; }}
        .dashboard-title {{ font-size: 20px; font-weight: 600; }}
        .header-actions {{ display: flex; gap: 12px; }}
        .btn {{ padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; }}
        .btn-primary {{ background: #0066cc; color: white; }}
        .btn-outline {{ background: white; border: 1px solid #ddd; color: #333; }}
        .dashboard-container {{ padding: 24px; }}
        .widget-grid {{ display: grid; grid-template-columns: repeat(12, 1fr); gap: 16px; min-height: calc(100vh - 150px); }}
        .widget {{ background: white; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); padding: 16px; position: relative; }}
        .widget-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }}
        .widget-title {{ font-weight: 600; font-size: 14px; }}
        .widget-menu {{ cursor: pointer; color: #999; }}
        .widget-content {{ height: calc(100% - 40px); display: flex; align-items: center; justify-content: center; color: #999; }}
        .empty-dashboard {{ text-align: center; padding: 80px; color: #666; grid-column: span 12; }}
        .empty-dashboard h3 {{ margin-bottom: 8px; color: #1a1a1a; }}
        .add-widget-btn {{ margin-top: 16px; padding: 12px 24px; background: #0066cc; color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }}
        .kpi-value {{ font-size: 36px; font-weight: 600; color: #1a1a1a; }}
        .kpi-label {{ font-size: 13px; color: #666; margin-top: 4px; }}
        .kpi-change {{ font-size: 13px; margin-top: 8px; }}
        .kpi-change.positive {{ color: #2e7d32; }}
        .kpi-change.negative {{ color: #c62828; }}
    </style>
</head>
<body>
    <div class="header">
        <div class="header-left">
            <a href="/suite/dashboards" class="back-link">←</a>
            <h1 class="dashboard-title" id="dashboardTitle">Loading...</h1>
        </div>
        <div class="header-actions">
            <button class="btn btn-outline" onclick="refreshData()">🔄 Refresh</button>
            <button class="btn btn-outline" onclick="shareDashboard()">🔗 Share</button>
            <button class="btn btn-primary" onclick="addWidget()">+ Add Widget</button>
            <button class="btn btn-outline" onclick="editDashboard()">⚙️ Settings</button>
        </div>
    </div>
    <div class="dashboard-container">
        <div class="widget-grid" id="widgetGrid">
            <div class="empty-dashboard">
                <h3>This dashboard is empty</h3>
                <p>Add widgets to start visualizing your data</p>
                <button class="add-widget-btn" onclick="addWidget()">+ Add Widget</button>
            </div>
        </div>
    </div>
    <script>
        const dashboardId = '{dashboard_id}';

        async function loadDashboard() {{
            try {{
                const response = await fetch(`/api/dashboards/${{dashboardId}}`);
                const dashboard = await response.json();
                if (dashboard) {{
                    document.getElementById('dashboardTitle').textContent = dashboard.name;
                    if (dashboard.widgets && dashboard.widgets.length > 0) {{
                        renderWidgets(dashboard.widgets);
                    }}
                }}
            }} catch (e) {{
                console.error('Failed to load dashboard:', e);
            }}
        }}

        function renderWidgets(widgets) {{
            const grid = document.getElementById('widgetGrid');
            grid.innerHTML = widgets.map(w => {{
                const colSpan = w.position?.width || 4;
                const rowSpan = w.position?.height || 2;
                return `
                    <div class="widget" style="grid-column: span ${{colSpan}}; grid-row: span ${{rowSpan}};">
                        <div class="widget-header">
                            <span class="widget-title">${{w.title}}</span>
                            <span class="widget-menu" onclick="widgetMenu('${{w.id}}')">⋮</span>
                        </div>
                        <div class="widget-content">
                            ${{renderWidgetContent(w)}}
                        </div>
                    </div>
                `;
            }}).join('');
        }}

        function renderWidgetContent(widget) {{
            switch (widget.widget_type) {{
                case 'kpi':
                    return `
                        <div style="text-align: center;">
                            <div class="kpi-value">${{widget.data?.value || '0'}}</div>
                            <div class="kpi-label">${{widget.config?.kpi_config?.value_field || 'Value'}}</div>
                            <div class="kpi-change positive">+12.5%</div>
                        </div>
                    `;
                case 'line_chart':
                case 'bar_chart':
                case 'pie_chart':
                    return `<div style="color: #999;">📊 ${{widget.widget_type.replace('_', ' ')}}</div>`;
                case 'table':
                    return `<div style="color: #999;">📋 Data table</div>`;
                default:
                    return `<div style="color: #999;">${{widget.widget_type || 'Widget'}}</div>`;
            }}
        }}

        function addWidget() {{
            window.location = `/suite/dashboards/${{dashboardId}}/widgets/new`;
        }}

        function editDashboard() {{
            window.location = `/suite/dashboards/${{dashboardId}}/edit`;
        }}

        function shareDashboard() {{
            navigator.clipboard.writeText(window.location.href);
            alert('Dashboard link copied to clipboard!');
        }}

        function refreshData() {{
            loadDashboard();
        }}

        function widgetMenu(widgetId) {{
            if (confirm('Delete this widget?')) {{
                fetch(`/api/dashboards/${{dashboardId}}/widgets/${{widgetId}}`, {{ method: 'DELETE' }})
                    .then(() => loadDashboard());
            }}
        }}

        loadDashboard();
    </script>
</body>
</html>"#);
    Html(html)
}

pub async fn handle_dashboard_new_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Create Dashboard</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .form-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group input, .form-group textarea { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
        .form-group textarea { min-height: 100px; resize: vertical; }
        .checkbox-group { display: flex; align-items: center; gap: 8px; }
        .checkbox-group input { width: auto; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .btn-secondary { background: #f5f5f5; color: #333; }
        .form-actions { display: flex; gap: 12px; justify-content: flex-end; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/dashboards" class="back-link">← Back to Dashboards</a>
        <div class="form-card">
            <h1>Create New Dashboard</h1>
            <form id="dashboardForm">
                <div class="form-group">
                    <label>Dashboard Name</label>
                    <input type="text" id="name" required placeholder="Enter dashboard name">
                </div>
                <div class="form-group">
                    <label>Description</label>
                    <textarea id="description" placeholder="Describe the purpose of this dashboard"></textarea>
                </div>
                <div class="form-group">
                    <label>Tags (comma-separated)</label>
                    <input type="text" id="tags" placeholder="e.g., sales, marketing, weekly">
                </div>
                <div class="form-group">
                    <label class="checkbox-group">
                        <input type="checkbox" id="isPublic">
                        <span>Make this dashboard public</span>
                    </label>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="window.location='/suite/dashboards'">Cancel</button>
                    <button type="submit" class="btn btn-primary">Create Dashboard</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        document.getElementById('dashboardForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const data = {
                name: document.getElementById('name').value,
                description: document.getElementById('description').value || null,
                tags: document.getElementById('tags').value.split(',').map(t => t.trim()).filter(t => t),
                is_public: document.getElementById('isPublic').checked
            };

            try {
                const response = await fetch('/api/dashboards', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });

                if (response.ok) {
                    const dashboard = await response.json();
                    window.location = `/suite/dashboards/${dashboard.id}`;
                } else {
                    alert('Failed to create dashboard');
                }
            } catch (e) {
                alert('Error: ' + e.message);
            }
        });
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub fn configure_dashboards_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/dashboards", get(handle_dashboards_list_page))
        .route("/suite/dashboards/new", get(handle_dashboard_new_page))
        .route("/suite/dashboards/:id", get(handle_dashboard_detail_page))
}
