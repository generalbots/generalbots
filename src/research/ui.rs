use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

pub async fn handle_research_list_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Research Projects</title>
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
        .research-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 24px; }
        .research-card { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; }
        .research-card:hover { transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
        .research-status { display: inline-block; padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 500; margin-bottom: 12px; }
        .status-active { background: #e8f5e9; color: #2e7d32; }
        .status-completed { background: #e3f2fd; color: #1565c0; }
        .status-draft { background: #f5f5f5; color: #666; }
        .research-title { font-size: 18px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
        .research-description { font-size: 14px; color: #666; line-height: 1.5; margin-bottom: 16px; }
        .research-meta { display: flex; justify-content: space-between; align-items: center; font-size: 13px; color: #999; }
        .research-tags { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 12px; }
        .tag { padding: 4px 10px; background: #f0f0f0; border-radius: 4px; font-size: 12px; color: #666; }
        .filters { display: flex; gap: 12px; margin-bottom: 24px; }
        .search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; }
        .filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
        .empty-state { text-align: center; padding: 80px 24px; color: #666; }
        .empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Research Projects</h1>
            <button class="btn btn-primary" onclick="createProject()">New Project</button>
        </div>
        <div class="tabs">
            <button class="tab active" data-status="all">All Projects</button>
            <button class="tab" data-status="active">Active</button>
            <button class="tab" data-status="completed">Completed</button>
            <button class="tab" data-status="draft">Drafts</button>
        </div>
        <div class="filters">
            <input type="text" class="search-box" placeholder="Search research projects..." id="searchInput">
            <select class="filter-select" id="typeFilter">
                <option value="">All Types</option>
                <option value="market">Market Research</option>
                <option value="user">User Research</option>
                <option value="competitive">Competitive Analysis</option>
                <option value="technical">Technical Research</option>
            </select>
            <select class="filter-select" id="sortBy">
                <option value="updated">Recently Updated</option>
                <option value="created">Recently Created</option>
                <option value="name">Name A-Z</option>
            </select>
        </div>
        <div class="research-grid" id="researchGrid">
            <div class="empty-state">
                <h3>No research projects yet</h3>
                <p>Create your first research project to get started</p>
            </div>
        </div>
    </div>
    <script>
        let currentStatus = 'all';

        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                currentStatus = tab.dataset.status;
                loadProjects();
            });
        });

        async function loadProjects() {
            try {
                const response = await fetch('/api/ui/research/collections');
                const data = await response.json();
                const projects = data.collections || data || [];
                renderProjects(Array.isArray(projects) ? projects : []);
            } catch (e) {
                console.error('Failed to load projects:', e);
            }
        }

        function renderProjects(projects) {
            const grid = document.getElementById('researchGrid');
            let filtered = projects;

            if (currentStatus !== 'all') {
                filtered = projects.filter(p => p.status === currentStatus);
            }

            if (!filtered || filtered.length === 0) {
                grid.innerHTML = '<div class="empty-state"><h3>No research projects found</h3><p>Create a new project to get started</p></div>';
                return;
            }

            grid.innerHTML = filtered.map(p => `
                <div class="research-card" onclick="window.location='/suite/research/${p.id}'">
                    <span class="research-status status-${p.status || 'draft'}">${(p.status || 'draft').charAt(0).toUpperCase() + (p.status || 'draft').slice(1)}</span>
                    <div class="research-title">${p.title || p.name}</div>
                    <div class="research-description">${p.description || 'No description'}</div>
                    <div class="research-meta">
                        <span>${p.findings_count || 0} findings</span>
                        <span>Updated ${formatDate(p.updated_at)}</span>
                    </div>
                    ${p.tags && p.tags.length ? `<div class="research-tags">${p.tags.map(t => `<span class="tag">${t}</span>`).join('')}</div>` : ''}
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

        function createProject() {
            window.location = '/suite/research/new';
        }

        loadProjects();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_research_detail_page(
    State(_state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Research Project</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 24px; }}
        .back-link {{ color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }}
        .project-header {{ background: white; border-radius: 12px; padding: 32px; margin-bottom: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .project-title {{ font-size: 28px; font-weight: 600; margin-bottom: 12px; }}
        .project-meta {{ color: #666; margin-bottom: 16px; }}
        .project-description {{ line-height: 1.6; color: #444; }}
        .section {{ background: white; border-radius: 12px; padding: 24px; margin-bottom: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .section-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }}
        .section-title {{ font-size: 18px; font-weight: 600; }}
        .btn {{ padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; }}
        .btn-primary {{ background: #0066cc; color: white; }}
        .btn-outline {{ background: transparent; border: 1px solid #ddd; color: #333; }}
        .finding-card {{ border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; margin-bottom: 12px; }}
        .finding-title {{ font-weight: 600; margin-bottom: 8px; }}
        .finding-content {{ color: #666; font-size: 14px; line-height: 1.5; }}
        .finding-meta {{ font-size: 12px; color: #999; margin-top: 8px; }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }}
        .stat-card {{ background: white; border-radius: 8px; padding: 20px; text-align: center; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .stat-value {{ font-size: 32px; font-weight: 600; color: #0066cc; }}
        .stat-label {{ font-size: 13px; color: #666; margin-top: 4px; }}
        .empty-findings {{ text-align: center; padding: 40px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/research" class="back-link">← Back to Projects</a>
        <div class="project-header">
            <h1 class="project-title" id="projectTitle">Loading...</h1>
            <div class="project-meta" id="projectMeta"></div>
            <p class="project-description" id="projectDescription"></p>
        </div>
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-value" id="findingsCount">0</div>
                <div class="stat-label">Findings</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="sourcesCount">0</div>
                <div class="stat-label">Sources</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="insightsCount">0</div>
                <div class="stat-label">Insights</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="progressValue">0%</div>
                <div class="stat-label">Progress</div>
            </div>
        </div>
        <div class="section">
            <div class="section-header">
                <h2 class="section-title">Key Findings</h2>
                <button class="btn btn-primary" onclick="addFinding()">Add Finding</button>
            </div>
            <div id="findingsList">
                <div class="empty-findings">No findings yet. Add your first finding to get started.</div>
            </div>
        </div>
    </div>
    <script>
        const projectId = '{project_id}';

        async function loadProject() {{
            try {{
                const response = await fetch(`/api/research/${{projectId}}`);
                const project = await response.json();
                if (project) {{
                    document.getElementById('projectTitle').textContent = project.title || project.name;
                    document.getElementById('projectMeta').textContent = `Created ${{new Date(project.created_at).toLocaleDateString()}} • Status: ${{project.status || 'Draft'}}`;
                    document.getElementById('projectDescription').textContent = project.description || '';
                    document.getElementById('findingsCount').textContent = project.findings_count || 0;
                    document.getElementById('sourcesCount').textContent = project.sources_count || 0;
                    document.getElementById('insightsCount').textContent = project.insights_count || 0;
                    document.getElementById('progressValue').textContent = (project.progress || 0) + '%';

                    if (project.findings && project.findings.length > 0) {{
                        renderFindings(project.findings);
                    }}
                }}
            }} catch (e) {{
                console.error('Failed to load project:', e);
            }}
        }}

        function renderFindings(findings) {{
            const list = document.getElementById('findingsList');
            list.innerHTML = findings.map(f => `
                <div class="finding-card">
                    <div class="finding-title">${{f.title}}</div>
                    <div class="finding-content">${{f.content || f.description}}</div>
                    <div class="finding-meta">Added ${{new Date(f.created_at).toLocaleDateString()}}</div>
                </div>
            `).join('');
        }}

        function addFinding() {{
            const title = prompt('Finding title:');
            if (!title) return;
            const content = prompt('Finding description:');
            if (!content) return;

            fetch(`/api/research/${{projectId}}/findings`, {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ title, content }})
            }}).then(() => loadProject());
        }}

        loadProject();
    </script>
</body>
</html>"#);
    Html(html)
}

pub async fn handle_research_new_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>New Research Project</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .form-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group input, .form-group textarea, .form-group select { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
        .form-group textarea { min-height: 120px; resize: vertical; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .form-actions { display: flex; gap: 12px; justify-content: flex-end; }
        .btn-secondary { background: #f5f5f5; color: #333; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/research" class="back-link">← Back to Projects</a>
        <div class="form-card">
            <h1>New Research Project</h1>
            <form id="projectForm">
                <div class="form-group">
                    <label>Project Title</label>
                    <input type="text" id="title" required placeholder="Enter project title">
                </div>
                <div class="form-group">
                    <label>Description</label>
                    <textarea id="description" placeholder="Describe the research objectives and scope"></textarea>
                </div>
                <div class="form-group">
                    <label>Research Type</label>
                    <select id="type">
                        <option value="market">Market Research</option>
                        <option value="user">User Research</option>
                        <option value="competitive">Competitive Analysis</option>
                        <option value="technical">Technical Research</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Tags (comma-separated)</label>
                    <input type="text" id="tags" placeholder="e.g., Q1, product-launch, customer-feedback">
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="window.location='/suite/research'">Cancel</button>
                    <button type="submit" class="btn btn-primary">Create Project</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        document.getElementById('projectForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const data = {
                title: document.getElementById('title').value,
                description: document.getElementById('description').value,
                research_type: document.getElementById('type').value,
                tags: document.getElementById('tags').value.split(',').map(t => t.trim()).filter(t => t)
            };

            try {
                const response = await fetch('/api/ui/research/collections/new', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });

                if (response.ok) {
                    const project = await response.json();
                    window.location = `/suite/research/${project.id}`;
                } else {
                    alert('Failed to create project');
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

pub fn configure_research_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/research", get(handle_research_list_page))
        .route("/suite/research/new", get(handle_research_new_page))
        .route("/suite/research/:id", get(handle_research_detail_page))
}
