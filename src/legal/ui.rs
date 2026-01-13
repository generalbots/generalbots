use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

pub async fn handle_legal_list_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Legal Documents</title>
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
        .stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }
        .stat-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .stat-value { font-size: 28px; font-weight: 600; color: #1a1a1a; }
        .stat-label { font-size: 13px; color: #666; margin-top: 4px; }
        .document-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 24px; }
        .document-card { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; }
        .document-card:hover { transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
        .document-icon { width: 48px; height: 48px; border-radius: 8px; background: #e3f2fd; display: flex; align-items: center; justify-content: center; font-size: 24px; margin-bottom: 16px; }
        .document-title { font-size: 16px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
        .document-meta { font-size: 13px; color: #666; margin-bottom: 12px; }
        .document-status { display: inline-block; padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 500; }
        .status-active { background: #e8f5e9; color: #2e7d32; }
        .status-draft { background: #f5f5f5; color: #666; }
        .status-expired { background: #ffebee; color: #c62828; }
        .status-review { background: #fff3e0; color: #ef6c00; }
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
            <h1>Legal Documents</h1>
            <button class="btn btn-primary" onclick="createDocument()">+ New Document</button>
        </div>
        <div class="stats-row">
            <div class="stat-card">
                <div class="stat-value" id="totalDocs">0</div>
                <div class="stat-label">Total Documents</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="activeDocs">0</div>
                <div class="stat-label">Active</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="pendingReview">0</div>
                <div class="stat-label">Pending Review</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="expiringDocs">0</div>
                <div class="stat-label">Expiring Soon</div>
            </div>
        </div>
        <div class="tabs">
            <button class="tab active" data-type="all">All Documents</button>
            <button class="tab" data-type="policies">Policies</button>
            <button class="tab" data-type="contracts">Contracts</button>
            <button class="tab" data-type="agreements">Agreements</button>
            <button class="tab" data-type="consents">Consent Forms</button>
        </div>
        <div class="filters">
            <input type="text" class="search-box" placeholder="Search legal documents..." id="searchInput" oninput="filterDocuments()">
            <select class="filter-select" id="statusFilter" onchange="filterDocuments()">
                <option value="">All Status</option>
                <option value="active">Active</option>
                <option value="draft">Draft</option>
                <option value="review">Under Review</option>
                <option value="expired">Expired</option>
            </select>
            <select class="filter-select" id="sortBy" onchange="filterDocuments()">
                <option value="updated">Recently Updated</option>
                <option value="created">Recently Created</option>
                <option value="name">Name A-Z</option>
                <option value="expiry">Expiry Date</option>
            </select>
        </div>
        <div class="document-grid" id="documentGrid">
            <div class="empty-state">
                <h3>No legal documents yet</h3>
                <p>Create your first legal document to get started</p>
            </div>
        </div>
    </div>
    <script>
        let documents = [];
        let currentType = 'all';

        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                currentType = tab.dataset.type;
                filterDocuments();
            });
        });

        async function loadDocuments() {
            try {
                const response = await fetch('/api/legal/documents');
                documents = await response.json();
                renderDocuments(documents);
                updateStats();
            } catch (e) {
                console.error('Failed to load documents:', e);
            }
        }

        function renderDocuments(docs) {
            const grid = document.getElementById('documentGrid');
            if (!docs || docs.length === 0) {
                grid.innerHTML = '<div class="empty-state"><h3>No documents found</h3><p>Create a new document to get started</p></div>';
                return;
            }

            grid.innerHTML = docs.map(d => `
                <div class="document-card" onclick="openDocument('${d.id}')">
                    <div class="document-icon">${getDocIcon(d.document_type)}</div>
                    <div class="document-title">${d.title}</div>
                    <div class="document-meta">
                        ${d.document_type || 'Document'} • Version ${d.version || '1.0'} • Updated ${formatDate(d.updated_at)}
                    </div>
                    <span class="document-status status-${d.status || 'draft'}">${(d.status || 'draft').charAt(0).toUpperCase() + (d.status || 'draft').slice(1)}</span>
                </div>
            `).join('');
        }

        function getDocIcon(type) {
            const icons = {
                policy: '📋',
                contract: '📝',
                agreement: '🤝',
                consent: '✅',
                terms: '📜',
                privacy: '🔒'
            };
            return icons[type] || '📄';
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

        function updateStats() {
            document.getElementById('totalDocs').textContent = documents.length;
            document.getElementById('activeDocs').textContent = documents.filter(d => d.status === 'active').length;
            document.getElementById('pendingReview').textContent = documents.filter(d => d.status === 'review').length;
            const now = new Date();
            const thirtyDays = 30 * 24 * 60 * 60 * 1000;
            document.getElementById('expiringDocs').textContent = documents.filter(d => {
                if (!d.expiry_date) return false;
                const expiry = new Date(d.expiry_date);
                return expiry - now < thirtyDays && expiry > now;
            }).length;
        }

        function filterDocuments() {
            const query = document.getElementById('searchInput').value.toLowerCase();
            const status = document.getElementById('statusFilter').value;

            let filtered = documents;

            if (currentType !== 'all') {
                filtered = filtered.filter(d => d.document_type === currentType.slice(0, -1));
            }

            if (query) {
                filtered = filtered.filter(d =>
                    d.title.toLowerCase().includes(query) ||
                    (d.description && d.description.toLowerCase().includes(query))
                );
            }

            if (status) {
                filtered = filtered.filter(d => d.status === status);
            }

            renderDocuments(filtered);
        }

        function createDocument() {
            window.location = '/suite/legal/new';
        }

        function openDocument(id) {
            window.location = `/suite/legal/${id}`;
        }

        loadDocuments();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_legal_detail_page(
    State(_state): State<Arc<AppState>>,
    Path(doc_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Legal Document</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
        .container {{ max-width: 1000px; margin: 0 auto; padding: 24px; }}
        .back-link {{ color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }}
        .document-header {{ background: white; border-radius: 12px; padding: 32px; margin-bottom: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .document-title {{ font-size: 28px; font-weight: 600; margin-bottom: 16px; }}
        .document-meta {{ display: flex; gap: 24px; color: #666; margin-bottom: 16px; flex-wrap: wrap; }}
        .document-status {{ display: inline-block; padding: 6px 16px; border-radius: 20px; font-size: 13px; font-weight: 500; }}
        .status-active {{ background: #e8f5e9; color: #2e7d32; }}
        .status-draft {{ background: #f5f5f5; color: #666; }}
        .document-actions {{ display: flex; gap: 12px; margin-top: 20px; }}
        .btn {{ padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }}
        .btn-primary {{ background: #0066cc; color: white; }}
        .btn-outline {{ background: white; border: 1px solid #ddd; color: #333; }}
        .btn-danger {{ background: #ffebee; color: #c62828; }}
        .document-content {{ background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .content-body {{ line-height: 1.8; color: #333; }}
        .content-body h2 {{ margin: 24px 0 12px; font-size: 20px; }}
        .content-body h3 {{ margin: 20px 0 10px; font-size: 16px; }}
        .content-body p {{ margin-bottom: 16px; }}
        .content-body ul, .content-body ol {{ margin-bottom: 16px; padding-left: 24px; }}
        .content-body li {{ margin-bottom: 8px; }}
        .version-history {{ background: white; border-radius: 12px; padding: 24px; margin-top: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .version-item {{ display: flex; justify-content: space-between; align-items: center; padding: 12px 0; border-bottom: 1px solid #f0f0f0; }}
        .version-item:last-child {{ border-bottom: none; }}
        .version-info {{ font-size: 14px; }}
        .version-date {{ color: #666; font-size: 13px; }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/legal" class="back-link">← Back to Legal Documents</a>
        <div class="document-header">
            <h1 class="document-title" id="docTitle">Loading...</h1>
            <div class="document-meta">
                <span id="docType">Document</span>
                <span id="docVersion">Version 1.0</span>
                <span id="docUpdated">Updated: -</span>
                <span id="docExpiry"></span>
            </div>
            <span class="document-status status-draft" id="docStatus">Draft</span>
            <div class="document-actions">
                <button class="btn btn-primary" onclick="editDocument()">Edit Document</button>
                <button class="btn btn-outline" onclick="downloadPdf()">Download PDF</button>
                <button class="btn btn-outline" onclick="viewHistory()">Version History</button>
                <button class="btn btn-danger" onclick="deleteDocument()">Delete</button>
            </div>
        </div>
        <div class="document-content">
            <div class="content-body" id="docContent">
                <p>Loading document content...</p>
            </div>
        </div>
        <div class="version-history" id="versionHistory" style="display: none;">
            <h3 style="margin-bottom: 16px;">Version History</h3>
            <div id="versionList"></div>
        </div>
    </div>
    <script>
        const docId = '{doc_id}';

        async function loadDocument() {{
            try {{
                const response = await fetch(`/api/legal/documents/${{docId}}`);
                const doc = await response.json();
                if (doc) {{
                    document.getElementById('docTitle').textContent = doc.title;
                    document.getElementById('docType').textContent = doc.document_type || 'Document';
                    document.getElementById('docVersion').textContent = `Version ${{doc.version || '1.0'}}`;
                    document.getElementById('docUpdated').textContent = `Updated: ${{new Date(doc.updated_at).toLocaleDateString()}}`;

                    if (doc.expiry_date) {{
                        document.getElementById('docExpiry').textContent = `Expires: ${{new Date(doc.expiry_date).toLocaleDateString()}}`;
                    }}

                    const statusEl = document.getElementById('docStatus');
                    statusEl.textContent = (doc.status || 'draft').charAt(0).toUpperCase() + (doc.status || 'draft').slice(1);
                    statusEl.className = `document-status status-${{doc.status || 'draft'}}`;

                    document.getElementById('docContent').innerHTML = doc.content || '<p>No content available</p>';
                }}
            }} catch (e) {{
                console.error('Failed to load document:', e);
            }}
        }}

        function editDocument() {{
            window.location = `/suite/legal/${{docId}}/edit`;
        }}

        async function downloadPdf() {{
            window.open(`/api/legal/documents/${{docId}}/pdf`, '_blank');
        }}

        function viewHistory() {{
            const history = document.getElementById('versionHistory');
            history.style.display = history.style.display === 'none' ? 'block' : 'none';
        }}

        async function deleteDocument() {{
            if (!confirm('Are you sure you want to delete this document? This action cannot be undone.')) return;
            try {{
                await fetch(`/api/legal/documents/${{docId}}`, {{ method: 'DELETE' }});
                window.location = '/suite/legal';
            }} catch (e) {{
                alert('Failed to delete document');
            }}
        }}

        loadDocument();
    </script>
</body>
</html>"#);
    Html(html)
}

pub async fn handle_legal_new_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Create Legal Document</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 900px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .form-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group input, .form-group textarea, .form-group select { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
        .form-group textarea { min-height: 300px; resize: vertical; font-family: inherit; }
        .form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .btn-secondary { background: #f5f5f5; color: #333; }
        .form-actions { display: flex; gap: 12px; justify-content: flex-end; }
        .template-buttons { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 16px; }
        .template-btn { padding: 8px 16px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; font-size: 13px; }
        .template-btn:hover { background: #f5f5f5; border-color: #0066cc; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/legal" class="back-link">← Back to Legal Documents</a>
        <div class="form-card">
            <h1>Create Legal Document</h1>
            <form id="documentForm">
                <div class="form-row">
                    <div class="form-group">
                        <label>Document Type</label>
                        <select id="documentType" required>
                            <option value="">Select type...</option>
                            <option value="policy">Policy</option>
                            <option value="contract">Contract</option>
                            <option value="agreement">Agreement</option>
                            <option value="consent">Consent Form</option>
                            <option value="terms">Terms of Service</option>
                            <option value="privacy">Privacy Policy</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Status</label>
                        <select id="status">
                            <option value="draft">Draft</option>
                            <option value="review">Under Review</option>
                            <option value="active">Active</option>
                        </select>
                    </div>
                </div>
                <div class="form-group">
                    <label>Document Title</label>
                    <input type="text" id="title" required placeholder="Enter document title">
                </div>
                <div class="form-group">
                    <label>Description</label>
                    <input type="text" id="description" placeholder="Brief description of this document">
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label>Effective Date</label>
                        <input type="date" id="effectiveDate">
                    </div>
                    <div class="form-group">
                        <label>Expiry Date (optional)</label>
                        <input type="date" id="expiryDate">
                    </div>
                </div>
                <div class="form-group">
                    <label>Document Content</label>
                    <div class="template-buttons">
                        <button type="button" class="template-btn" onclick="insertTemplate('privacy')">Privacy Policy Template</button>
                        <button type="button" class="template-btn" onclick="insertTemplate('terms')">Terms of Service Template</button>
                        <button type="button" class="template-btn" onclick="insertTemplate('cookie')">Cookie Policy Template</button>
                    </div>
                    <textarea id="content" placeholder="Enter the document content here..."></textarea>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="saveDraft()">Save as Draft</button>
                    <button type="button" class="btn btn-secondary" onclick="window.location='/suite/legal'">Cancel</button>
                    <button type="submit" class="btn btn-primary">Create Document</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        const templates = {
            privacy: `<h2>Privacy Policy</h2>
<p>Last updated: [DATE]</p>

<h3>1. Information We Collect</h3>
<p>We collect information you provide directly to us, including...</p>

<h3>2. How We Use Your Information</h3>
<p>We use the information we collect to...</p>

<h3>3. Information Sharing</h3>
<p>We do not share your personal information except...</p>

<h3>4. Data Security</h3>
<p>We implement appropriate security measures to protect...</p>

<h3>5. Your Rights</h3>
<p>You have the right to access, correct, or delete your personal data...</p>

<h3>6. Contact Us</h3>
<p>If you have questions about this Privacy Policy, please contact us at...</p>`,
            terms: `<h2>Terms of Service</h2>
<p>Last updated: [DATE]</p>

<h3>1. Acceptance of Terms</h3>
<p>By accessing or using our services, you agree to be bound by these Terms...</p>

<h3>2. Use of Services</h3>
<p>You may use our services only in compliance with these Terms...</p>

<h3>3. User Accounts</h3>
<p>You are responsible for maintaining the confidentiality of your account...</p>

<h3>4. Intellectual Property</h3>
<p>All content and materials available through our services are protected...</p>

<h3>5. Limitation of Liability</h3>
<p>To the fullest extent permitted by law, we shall not be liable...</p>

<h3>6. Governing Law</h3>
<p>These Terms shall be governed by and construed in accordance with...</p>`,
            cookie: `<h2>Cookie Policy</h2>
<p>Last updated: [DATE]</p>

<h3>1. What Are Cookies</h3>
<p>Cookies are small text files stored on your device when you visit our website...</p>

<h3>2. Types of Cookies We Use</h3>
<ul>
<li><strong>Essential Cookies:</strong> Required for basic site functionality</li>
<li><strong>Analytics Cookies:</strong> Help us understand how visitors use our site</li>
<li><strong>Marketing Cookies:</strong> Used to deliver relevant advertisements</li>
</ul>

<h3>3. Managing Cookies</h3>
<p>You can control cookies through your browser settings...</p>

<h3>4. Contact Us</h3>
<p>For questions about our Cookie Policy, contact us at...</p>`
        };

        function insertTemplate(type) {
            const content = document.getElementById('content');
            const today = new Date().toLocaleDateString();
            content.value = templates[type].replace('[DATE]', today);
        }

        function saveDraft() {
            document.getElementById('status').value = 'draft';
            document.getElementById('documentForm').dispatchEvent(new Event('submit'));
        }

        document.getElementById('documentForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const data = {
                document_type: document.getElementById('documentType').value,
                title: document.getElementById('title').value,
                description: document.getElementById('description').value || null,
                content: document.getElementById('content').value,
                status: document.getElementById('status').value,
                effective_date: document.getElementById('effectiveDate').value || null,
                expiry_date: document.getElementById('expiryDate').value || null
            };

            try {
                const response = await fetch('/api/legal/documents', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });

                if (response.ok) {
                    const doc = await response.json();
                    window.location = `/suite/legal/${doc.id}`;
                } else {
                    alert('Failed to create document');
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

pub fn configure_legal_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/legal", get(handle_legal_list_page))
        .route("/suite/legal/new", get(handle_legal_new_page))
        .route("/suite/legal/:id", get(handle_legal_detail_page))
}
