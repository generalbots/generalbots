use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

pub async fn handle_email_inbox_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Email Inbox</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; height: 100vh; display: flex; }
        .sidebar { width: 240px; background: white; border-right: 1px solid #e0e0e0; padding: 16px; flex-shrink: 0; }
        .sidebar h2 { font-size: 20px; margin-bottom: 20px; padding: 8px; }
        .compose-btn { width: 100%; padding: 14px 20px; background: #0066cc; color: white; border: none; border-radius: 24px; font-size: 14px; font-weight: 500; cursor: pointer; margin-bottom: 20px; }
        .compose-btn:hover { background: #0052a3; }
        .nav-item { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; cursor: pointer; color: #333; text-decoration: none; margin-bottom: 4px; }
        .nav-item:hover { background: #f5f5f5; }
        .nav-item.active { background: #e8f4ff; color: #0066cc; font-weight: 500; }
        .nav-item .count { margin-left: auto; background: #e0e0e0; padding: 2px 8px; border-radius: 12px; font-size: 12px; }
        .nav-item.active .count { background: #0066cc; color: white; }
        .main-content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
        .toolbar { display: flex; align-items: center; gap: 12px; padding: 12px 20px; background: white; border-bottom: 1px solid #e0e0e0; }
        .search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
        .toolbar-btn { padding: 8px 12px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; }
        .email-list { flex: 1; overflow-y: auto; background: white; }
        .email-item { display: flex; align-items: center; padding: 16px 20px; border-bottom: 1px solid #f0f0f0; cursor: pointer; transition: background 0.15s; }
        .email-item:hover { background: #f8f9fa; }
        .email-item.unread { background: #f0f7ff; }
        .email-item.unread:hover { background: #e8f4ff; }
        .email-item.selected { background: #e3f2fd; }
        .email-checkbox { margin-right: 16px; }
        .email-star { margin-right: 12px; color: #ddd; cursor: pointer; font-size: 18px; }
        .email-star.starred { color: #ffc107; }
        .email-sender { width: 200px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .email-item.unread .email-sender { font-weight: 600; }
        .email-content { flex: 1; display: flex; gap: 8px; overflow: hidden; }
        .email-subject { font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .email-item.unread .email-subject { font-weight: 600; }
        .email-preview { color: #666; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .email-date { width: 100px; text-align: right; font-size: 13px; color: #666; flex-shrink: 0; }
        .email-item.unread .email-date { font-weight: 500; color: #333; }
        .empty-state { text-align: center; padding: 80px 24px; color: #666; }
        .empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
        .pagination { display: flex; justify-content: space-between; align-items: center; padding: 12px 20px; background: white; border-top: 1px solid #e0e0e0; font-size: 13px; color: #666; }
    </style>
</head>
<body>
    <div class="sidebar">
        <h2>Mail</h2>
        <button class="compose-btn" onclick="composeMail()">✏️ Compose</button>
        <a href="/suite/email" class="nav-item active">
            <span>📥</span> Inbox <span class="count" id="inboxCount">0</span>
        </a>
        <a href="/suite/email/starred" class="nav-item">
            <span>⭐</span> Starred
        </a>
        <a href="/suite/email/sent" class="nav-item">
            <span>📤</span> Sent
        </a>
        <a href="/suite/email/drafts" class="nav-item">
            <span>📝</span> Drafts <span class="count" id="draftsCount">0</span>
        </a>
        <a href="/suite/email/archive" class="nav-item">
            <span>📁</span> Archive
        </a>
        <a href="/suite/email/spam" class="nav-item">
            <span>🚫</span> Spam
        </a>
        <a href="/suite/email/trash" class="nav-item">
            <span>🗑️</span> Trash
        </a>
    </div>
    <div class="main-content">
        <div class="toolbar">
            <input type="checkbox" id="selectAll" onclick="toggleSelectAll()">
            <button class="toolbar-btn" onclick="archiveSelected()" title="Archive">📁</button>
            <button class="toolbar-btn" onclick="deleteSelected()" title="Delete">🗑️</button>
            <button class="toolbar-btn" onclick="markAsRead()" title="Mark as read">✉️</button>
            <input type="text" class="search-box" placeholder="Search emails..." id="searchInput" oninput="searchEmails()">
            <button class="toolbar-btn" onclick="refreshInbox()">🔄</button>
        </div>
        <div class="email-list" id="emailList">
            <div class="empty-state">
                <h3>Your inbox is empty</h3>
                <p>Emails you receive will appear here</p>
            </div>
        </div>
        <div class="pagination">
            <span id="paginationInfo">0 emails</span>
            <div>
                <button class="toolbar-btn" onclick="prevPage()" id="prevBtn" disabled>← Prev</button>
                <button class="toolbar-btn" onclick="nextPage()" id="nextBtn" disabled>Next →</button>
            </div>
        </div>
    </div>
    <script>
        let emails = [];
        let selectedEmails = new Set();
        let currentPage = 1;
        const pageSize = 50;

        async function loadEmails() {
            try {
                const response = await fetch('/api/email/list', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ folder: currentFolder })
                });
                const data = await response.json();
                emails = data.emails || data || [];
                renderEmails();
                updateCounts();
            } catch (e) {
                console.error('Failed to load emails:', e);
            }
        }

        function renderEmails() {
            const list = document.getElementById('emailList');
            if (!emails || emails.length === 0) {
                list.innerHTML = '<div class="empty-state"><h3>Your inbox is empty</h3><p>Emails you receive will appear here</p></div>';
                return;
            }

            list.innerHTML = emails.map(e => `
                <div class="email-item ${e.is_read ? '' : 'unread'} ${selectedEmails.has(e.id) ? 'selected' : ''}" onclick="openEmail('${e.id}')">
                    <input type="checkbox" class="email-checkbox" ${selectedEmails.has(e.id) ? 'checked' : ''} onclick="event.stopPropagation(); toggleSelect('${e.id}')">
                    <span class="email-star ${e.is_starred ? 'starred' : ''}" onclick="event.stopPropagation(); toggleStar('${e.id}')">${e.is_starred ? '★' : '☆'}</span>
                    <div class="email-sender">${e.from_name || e.from_address}</div>
                    <div class="email-content">
                        <span class="email-subject">${e.subject || '(No subject)'}</span>
                        <span class="email-preview"> - ${e.preview || e.body_text || ''}</span>
                    </div>
                    <div class="email-date">${formatDate(e.received_at || e.created_at)}</div>
                </div>
            `).join('');

            document.getElementById('paginationInfo').textContent = `${emails.length} emails`;
        }

        function formatDate(dateStr) {
            if (!dateStr) return '';
            const date = new Date(dateStr);
            const now = new Date();
            const diff = now - date;
            if (diff < 86400000) {
                return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
            }
            if (diff < 604800000) {
                return date.toLocaleDateString([], { weekday: 'short' });
            }
            return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
        }

        function updateCounts() {
            const unread = emails.filter(e => !e.is_read).length;
            document.getElementById('inboxCount').textContent = unread || '';
        }

        function openEmail(id) {
            window.location = `/suite/email/${id}`;
        }

        function composeMail() {
            window.location = '/suite/email/compose';
        }

        function toggleSelect(id) {
            if (selectedEmails.has(id)) {
                selectedEmails.delete(id);
            } else {
                selectedEmails.add(id);
            }
            renderEmails();
        }

        function toggleSelectAll() {
            const selectAll = document.getElementById('selectAll').checked;
            if (selectAll) {
                emails.forEach(e => selectedEmails.add(e.id));
            } else {
                selectedEmails.clear();
            }
            renderEmails();
        }

        async function toggleStar(id) {
            const email = emails.find(e => e.id === id);
            if (email) {
                email.is_starred = !email.is_starred;
                await fetch(`/api/email/messages/${id}/star`, { method: 'POST' });
                renderEmails();
            }
        }

        async function archiveSelected() {
            if (selectedEmails.size === 0) return;
            for (const id of selectedEmails) {
                await fetch(`/api/email/messages/${id}/archive`, { method: 'POST' });
            }
            selectedEmails.clear();
            loadEmails();
        }

        async function deleteSelected() {
            if (selectedEmails.size === 0) return;
            if (!confirm(`Delete ${selectedEmails.size} email(s)?`)) return;
            for (const id of selectedEmails) {
                await fetch(`/api/email/messages/${id}`, { method: 'DELETE' });
            }
            selectedEmails.clear();
            loadEmails();
        }

        async function markAsRead() {
            if (selectedEmails.size === 0) return;
            for (const id of selectedEmails) {
                await fetch(`/api/email/messages/${id}/read`, { method: 'POST' });
            }
            loadEmails();
        }

        function searchEmails() {
            const query = document.getElementById('searchInput').value.toLowerCase();
            const filtered = emails.filter(e =>
                (e.subject && e.subject.toLowerCase().includes(query)) ||
                (e.from_name && e.from_name.toLowerCase().includes(query)) ||
                (e.from_address && e.from_address.toLowerCase().includes(query)) ||
                (e.body_text && e.body_text.toLowerCase().includes(query))
            );
            renderFilteredEmails(filtered);
        }

        function renderFilteredEmails(filtered) {
            const list = document.getElementById('emailList');
            if (!filtered || filtered.length === 0) {
                list.innerHTML = '<div class="empty-state"><h3>No emails found</h3><p>Try a different search term</p></div>';
                return;
            }
            list.innerHTML = filtered.map(e => `
                <div class="email-item ${e.is_read ? '' : 'unread'}" onclick="openEmail('${e.id}')">
                    <input type="checkbox" class="email-checkbox" onclick="event.stopPropagation(); toggleSelect('${e.id}')">
                    <span class="email-star ${e.is_starred ? 'starred' : ''}" onclick="event.stopPropagation(); toggleStar('${e.id}')">${e.is_starred ? '★' : '☆'}</span>
                    <div class="email-sender">${e.from_name || e.from_address}</div>
                    <div class="email-content">
                        <span class="email-subject">${e.subject || '(No subject)'}</span>
                        <span class="email-preview"> - ${e.preview || ''}</span>
                    </div>
                    <div class="email-date">${formatDate(e.received_at)}</div>
                </div>
            `).join('');
        }

        function refreshInbox() {
            loadEmails();
        }

        loadEmails();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_email_detail_page(
    State(_state): State<Arc<AppState>>,
    Path(email_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Email</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
        .container {{ max-width: 900px; margin: 0 auto; padding: 24px; }}
        .back-link {{ color: #0066cc; text-decoration: none; display: inline-flex; align-items: center; gap: 8px; margin-bottom: 16px; }}
        .email-card {{ background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); overflow: hidden; }}
        .email-header {{ padding: 24px; border-bottom: 1px solid #e0e0e0; }}
        .email-subject {{ font-size: 24px; font-weight: 600; margin-bottom: 16px; }}
        .email-meta {{ display: flex; align-items: flex-start; gap: 16px; }}
        .sender-avatar {{ width: 48px; height: 48px; border-radius: 50%; background: #0066cc; color: white; display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: 500; flex-shrink: 0; }}
        .sender-info {{ flex: 1; }}
        .sender-name {{ font-weight: 600; font-size: 16px; }}
        .sender-email {{ color: #666; font-size: 14px; }}
        .email-date {{ color: #666; font-size: 14px; }}
        .email-recipients {{ margin-top: 8px; font-size: 13px; color: #666; }}
        .email-actions {{ display: flex; gap: 8px; }}
        .action-btn {{ padding: 8px 16px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; font-size: 14px; }}
        .action-btn:hover {{ background: #f5f5f5; }}
        .action-btn.primary {{ background: #0066cc; color: white; border-color: #0066cc; }}
        .action-btn.primary:hover {{ background: #0052a3; }}
        .email-body {{ padding: 24px; line-height: 1.7; font-size: 15px; }}
        .email-body p {{ margin-bottom: 16px; }}
        .attachments {{ padding: 16px 24px; background: #f9f9f9; border-top: 1px solid #e0e0e0; }}
        .attachments-title {{ font-weight: 600; margin-bottom: 12px; font-size: 14px; }}
        .attachment {{ display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; background: white; border: 1px solid #ddd; border-radius: 6px; margin-right: 8px; margin-bottom: 8px; cursor: pointer; }}
        .attachment:hover {{ background: #f5f5f5; }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/email" class="back-link">← Back to Inbox</a>
        <div class="email-card">
            <div class="email-header">
                <h1 class="email-subject" id="emailSubject">Loading...</h1>
                <div class="email-meta">
                    <div class="sender-avatar" id="senderAvatar">?</div>
                    <div class="sender-info">
                        <div class="sender-name" id="senderName">Loading...</div>
                        <div class="sender-email" id="senderEmail"></div>
                        <div class="email-recipients" id="recipients"></div>
                    </div>
                    <div style="text-align: right;">
                        <div class="email-date" id="emailDate"></div>
                        <div class="email-actions" style="margin-top: 12px;">
                            <button class="action-btn primary" onclick="replyEmail()">↩️ Reply</button>
                            <button class="action-btn" onclick="forwardEmail()">↪️ Forward</button>
                            <button class="action-btn" onclick="deleteEmail()">🗑️ Delete</button>
                        </div>
                    </div>
                </div>
            </div>
            <div class="email-body" id="emailBody">
                <p>Loading email content...</p>
            </div>
            <div class="attachments" id="attachments" style="display: none;">
                <div class="attachments-title">📎 Attachments</div>
                <div id="attachmentsList"></div>
            </div>
        </div>
    </div>
    <script>
        const emailId = '{email_id}';

        async function loadEmail() {{
            try {{
                const response = await fetch(`/api/email/messages/${{emailId}}`);
                const email = await response.json();
                if (email) {{
                    document.getElementById('emailSubject').textContent = email.subject || '(No subject)';
                    document.getElementById('senderName').textContent = email.from_name || email.from_address;
                    document.getElementById('senderEmail').textContent = email.from_address ? `<${{email.from_address}}>` : '';
                    document.getElementById('senderAvatar').textContent = (email.from_name || email.from_address || '?')[0].toUpperCase();
                    document.getElementById('emailDate').textContent = email.received_at ? new Date(email.received_at).toLocaleString() : '';

                    if (email.to_addresses && email.to_addresses.length) {{
                        document.getElementById('recipients').textContent = `To: ${{email.to_addresses.join(', ')}}`;
                    }}

                    const body = email.body_html || email.body_text || 'No content';
                    document.getElementById('emailBody').innerHTML = email.body_html ? body : `<p>${{body.replace(/\\n/g, '</p><p>')}}</p>`;

                    if (email.attachments && email.attachments.length) {{
                        document.getElementById('attachments').style.display = 'block';
                        document.getElementById('attachmentsList').innerHTML = email.attachments.map(a => `
                            <div class="attachment" onclick="downloadAttachment('${{a.id}}')">
                                📄 ${{a.filename}} (${{formatSize(a.size)}})
                            </div>
                        `).join('');
                    }}

                    if (!email.is_read) {{
                        fetch(`/api/email/messages/${{emailId}}/read`, {{ method: 'POST' }});
                    }}
                }}
            }} catch (e) {{
                console.error('Failed to load email:', e);
                document.getElementById('emailBody').innerHTML = '<p>Failed to load email content</p>';
            }}
        }}

        function formatSize(bytes) {{
            if (!bytes) return '0 B';
            const sizes = ['B', 'KB', 'MB', 'GB'];
            const i = Math.floor(Math.log(bytes) / Math.log(1024));
            return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + sizes[i];
        }}

        function replyEmail() {{
            window.location = `/suite/email/compose?reply=${{emailId}}`;
        }}

        function forwardEmail() {{
            window.location = `/suite/email/compose?forward=${{emailId}}`;
        }}

        async function deleteEmail() {{
            if (!confirm('Move this email to trash?')) return;
            try {{
                await fetch(`/api/email/messages/${{emailId}}`, {{ method: 'DELETE' }});
                window.location = '/suite/email';
            }} catch (e) {{
                alert('Failed to delete email');
            }}
        }}

        function downloadAttachment(attachmentId) {{
            window.open(`/api/email/attachments/${{attachmentId}}/download`, '_blank');
        }}

        loadEmail();
    </script>
</body>
</html>"#
    );
    Html(html)
}

pub async fn handle_email_compose_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Compose Email</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 900px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-flex; align-items: center; gap: 8px; margin-bottom: 16px; }
        .compose-card { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); overflow: hidden; }
        .compose-header { padding: 16px 24px; border-bottom: 1px solid #e0e0e0; display: flex; justify-content: space-between; align-items: center; }
        .compose-header h1 { font-size: 20px; }
        .compose-form { padding: 0; }
        .form-row { display: flex; align-items: center; border-bottom: 1px solid #e0e0e0; }
        .form-label { width: 80px; padding: 12px 24px; font-weight: 500; color: #666; flex-shrink: 0; }
        .form-input { flex: 1; padding: 12px 16px; border: none; font-size: 14px; outline: none; }
        .form-input:focus { background: #f8fafc; }
        .body-editor { padding: 24px; min-height: 400px; }
        .body-editor textarea { width: 100%; min-height: 350px; border: none; font-size: 15px; line-height: 1.6; resize: vertical; outline: none; }
        .compose-footer { padding: 16px 24px; border-top: 1px solid #e0e0e0; display: flex; justify-content: space-between; align-items: center; background: #f9f9f9; }
        .btn { padding: 10px 24px; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .btn-secondary { background: white; border: 1px solid #ddd; color: #333; }
        .btn-secondary:hover { background: #f5f5f5; }
        .footer-actions { display: flex; gap: 12px; }
        .attachment-btn { display: inline-flex; align-items: center; gap: 8px; padding: 8px 16px; background: white; border: 1px solid #ddd; border-radius: 6px; cursor: pointer; }
        .attachment-btn:hover { background: #f5f5f5; }
        .attachments-list { padding: 0 24px 16px; display: flex; flex-wrap: wrap; gap: 8px; }
        .attachment-item { display: inline-flex; align-items: center; gap: 8px; padding: 6px 12px; background: #f5f5f5; border-radius: 4px; font-size: 13px; }
        .attachment-remove { cursor: pointer; color: #999; }
        .attachment-remove:hover { color: #c62828; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/email" class="back-link">← Back to Inbox</a>
        <div class="compose-card">
            <div class="compose-header">
                <h1>New Message</h1>
            </div>
            <form id="composeForm" class="compose-form">
                <div class="form-row">
                    <label class="form-label">To</label>
                    <input type="text" class="form-input" id="toField" placeholder="Recipients" required>
                </div>
                <div class="form-row">
                    <label class="form-label">Cc</label>
                    <input type="text" class="form-input" id="ccField" placeholder="Cc recipients">
                </div>
                <div class="form-row">
                    <label class="form-label">Subject</label>
                    <input type="text" class="form-input" id="subjectField" placeholder="Subject">
                </div>
                <div class="attachments-list" id="attachmentsList"></div>
                <div class="body-editor">
                    <textarea id="bodyField" placeholder="Write your message..."></textarea>
                </div>
                <div class="compose-footer">
                    <div>
                        <label class="attachment-btn">
                            📎 Attach files
                            <input type="file" id="attachmentInput" multiple style="display: none;">
                        </label>
                    </div>
                    <div class="footer-actions">
                        <button type="button" class="btn btn-secondary" onclick="saveDraft()">Save Draft</button>
                        <button type="button" class="btn btn-secondary" onclick="discardDraft()">Discard</button>
                        <button type="submit" class="btn btn-primary">Send</button>
                    </div>
                </div>
            </form>
        </div>
    </div>
    <script>
        let attachments = [];

        document.getElementById('attachmentInput').addEventListener('change', (e) => {
            for (const file of e.target.files) {
                attachments.push(file);
            }
            renderAttachments();
        });

        function renderAttachments() {
            const list = document.getElementById('attachmentsList');
            list.innerHTML = attachments.map((f, i) => `
                <div class="attachment-item">
                    📄 ${f.name}
                    <span class="attachment-remove" onclick="removeAttachment(${i})">✕</span>
                </div>
            `).join('');
        }

        function removeAttachment(index) {
            attachments.splice(index, 1);
            renderAttachments();
        }

        async function saveDraft() {
            const data = {
                to_addresses: document.getElementById('toField').value.split(',').map(e => e.trim()).filter(e => e),
                cc_addresses: document.getElementById('ccField').value.split(',').map(e => e.trim()).filter(e => e),
                subject: document.getElementById('subjectField').value,
                body_text: document.getElementById('bodyField').value,
                is_draft: true
            };

            try {
                await fetch('/api/email/send', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });
                window.location = '/suite/email/drafts';
            } catch (e) {
                alert('Failed to save draft');
            }
        }

        function discardDraft() {
            if (document.getElementById('bodyField').value && !confirm('Discard this draft?')) return;
            window.location = '/suite/email';
        }

        document.getElementById('composeForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const toField = document.getElementById('toField').value;
            if (!toField.trim()) {
                alert('Please enter at least one recipient');
                return;
            }

            const data = {
                to_addresses: toField.split(',').map(e => e.trim()).filter(e => e),
                cc_addresses: document.getElementById('ccField').value.split(',').map(e => e.trim()).filter(e => e),
                subject: document.getElementById('subjectField').value,
                body_text: document.getElementById('bodyField').value,
                is_draft: false
            };

            try {
                await fetch('/api/email/draft', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });
                window.location = '/suite/email';
            } catch (e) {
                alert('Failed to send email');
            }
        });

        const params = new URLSearchParams(window.location.search);
        if (params.get('reply') || params.get('forward')) {
            loadReplyData(params.get('reply') || params.get('forward'), !!params.get('forward'));
        }

        async function loadReplyData(emailId, isForward) {
            try {
                const response = await fetch(`/api/email/messages/${emailId}`);
                const email = await response.json();
                if (email) {
                    if (!isForward) {
                        document.getElementById('toField').value = email.from_address || '';
                        document.getElementById('subjectField').value = 'Re: ' + (email.subject || '');
                    } else {
                        document.getElementById('subjectField').value = 'Fwd: ' + (email.subject || '');
                    }
                    const quote = `\n\n--- Original Message ---\nFrom: ${email.from_address}\nDate: ${new Date(email.received_at).toLocaleString()}\nSubject: ${email.subject}\n\n${email.body_text || ''}`;
                    document.getElementById('bodyField').value = quote;
                }
            } catch (e) {
                console.error('Failed to load reply data:', e);
            }
        }
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub fn configure_email_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/email", get(handle_email_inbox_page))
        .route("/suite/email/compose", get(handle_email_compose_page))
        .route("/suite/email/:id", get(handle_email_detail_page))
}
