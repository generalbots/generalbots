use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

pub async fn handle_designer_list_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dialog Designer</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
        .header h1 { font-size: 28px; color: #1a1a1a; }
        .btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .dialog-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 20px; }
        .dialog-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; }
        .dialog-card:hover { box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
        .dialog-icon { width: 48px; height: 48px; background: #e8f4ff; border-radius: 10px; display: flex; align-items: center; justify-content: center; margin-bottom: 12px; font-size: 24px; }
        .dialog-name { font-size: 16px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
        .dialog-meta { font-size: 12px; color: #999; }
        .empty-state { text-align: center; padding: 80px 24px; color: #666; }
        .empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
        .search-box { padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; width: 300px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Dialog Designer</h1>
            <div style="display: flex; gap: 12px;">
                <input type="text" class="search-box" placeholder="Search dialogs..." id="searchInput">
                <button class="btn btn-primary" onclick="window.location='/suite/designer/new'">New Dialog</button>
            </div>
        </div>
        <div class="dialog-grid" id="dialogGrid">
            <div class="empty-state"><h3>Loading...</h3></div>
        </div>
    </div>
    <script>
        async function loadDialogs() {
            try {
                const response = await fetch('/api/ui/designer/dialogs');
                const data = await response.json();
                renderDialogs(data.dialogs || data || []);
            } catch (e) {
                document.getElementById('dialogGrid').innerHTML = '<div class="empty-state"><h3>No dialogs yet</h3><p>Create your first dialog</p></div>';
            }
        }
        function renderDialogs(dialogs) {
            const grid = document.getElementById('dialogGrid');
            if (!dialogs.length) {
                grid.innerHTML = '<div class="empty-state"><h3>No dialogs yet</h3><p>Create your first dialog</p></div>';
                return;
            }
            grid.innerHTML = dialogs.map(d => `
                <div class="dialog-card" onclick="window.location='/suite/designer/edit/${d.id}'">
                    <div class="dialog-icon">💬</div>
                    <div class="dialog-name">${d.name}</div>
                    <div class="dialog-meta">Updated ${new Date(d.updated_at).toLocaleDateString()}</div>
                </div>
            `).join('');
        }
        loadDialogs();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_designer_edit_page(
    State(_state): State<Arc<AppState>>,
    Path(dialog_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Edit Dialog</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #1e1e1e; color: #d4d4d4; height: 100vh; display: flex; flex-direction: column; }}
        .toolbar {{ background: #2d2d2d; padding: 12px 20px; display: flex; justify-content: space-between; align-items: center; }}
        .back-link {{ color: #0078d4; text-decoration: none; }}
        .btn {{ padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 13px; }}
        .btn-primary {{ background: #0078d4; color: white; }}
        .main {{ flex: 1; display: flex; }}
        .sidebar {{ width: 250px; background: #252526; padding: 16px; }}
        .canvas {{ flex: 1; background: #1e1e1e; position: relative; }}
        .properties {{ width: 300px; background: #252526; padding: 16px; }}
        .node-item {{ background: #2d2d2d; padding: 12px; margin-bottom: 8px; border-radius: 8px; cursor: grab; }}
        .section-title {{ font-size: 12px; color: #888; margin-bottom: 12px; }}
        .form-group {{ margin-bottom: 12px; }}
        .form-group label {{ display: block; font-size: 12px; color: #888; margin-bottom: 4px; }}
        .form-group input, .form-group textarea {{ width: 100%; padding: 8px; border: 1px solid #404040; background: #2d2d2d; color: #d4d4d4; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="toolbar">
        <a href="/suite/designer" class="back-link">← Back</a>
        <span id="dialogName">Loading...</span>
        <button class="btn btn-primary" onclick="saveDialog()">Save</button>
    </div>
    <div class="main">
        <div class="sidebar">
            <div class="section-title">NODES</div>
            <div class="node-item">💬 Message</div>
            <div class="node-item">❓ Question</div>
            <div class="node-item">🔀 Condition</div>
            <div class="node-item">⚡ Action</div>
        </div>
        <div class="canvas" id="canvas"></div>
        <div class="properties">
            <div class="section-title">PROPERTIES</div>
            <div id="propertiesContent">Select a node to edit</div>
        </div>
    </div>
    <script>
        const dialogId = '{dialog_id}';
        async function loadDialog() {{
            try {{
                const response = await fetch('/api/ui/designer/dialogs/' + dialogId);
                const data = await response.json();
                document.getElementById('dialogName').textContent = data.name || 'Untitled';
            }} catch (e) {{ console.error(e); }}
        }}
        async function saveDialog() {{
            alert('Save functionality - implement based on canvas state');
        }}
        loadDialog();
    </script>
</body>
</html>"#);
    Html(html)
}

pub async fn handle_designer_new_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Create Dialog</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 600px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group input, .form-group textarea { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/designer" class="back-link">← Back</a>
        <div class="card">
            <h1>Create New Dialog</h1>
            <form id="createForm">
                <div class="form-group">
                    <label>Name</label>
                    <input type="text" id="name" required placeholder="Dialog name">
                </div>
                <div class="form-group">
                    <label>Description</label>
                    <textarea id="description" placeholder="Description"></textarea>
                </div>
                <button type="submit" class="btn btn-primary">Create</button>
            </form>
        </div>
    </div>
    <script>
        document.getElementById('createForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            try {
                const response = await fetch('/api/ui/designer/dialogs', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        name: document.getElementById('name').value,
                        description: document.getElementById('description').value
                    })
                });
                const data = await response.json();
                if (data.id) window.location = '/suite/designer/edit/' + data.id;
            } catch (e) { alert('Error: ' + e.message); }
        });
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub fn configure_designer_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/designer", get(handle_designer_list_page))
        .route("/suite/designer/new", get(handle_designer_new_page))
        .route("/suite/designer/edit/:id", get(handle_designer_edit_page))
}
