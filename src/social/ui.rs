use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

pub async fn handle_social_list_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Social Media Manager</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
        .header h1 { font-size: 28px; color: #1a1a1a; }
        .btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }
        .stat-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .stat-value { font-size: 28px; font-weight: 600; color: #1a1a1a; }
        .stat-label { font-size: 13px; color: #666; margin-top: 4px; }
        .stat-change { font-size: 12px; margin-top: 8px; }
        .stat-change.positive { color: #2e7d32; }
        .stat-change.negative { color: #c62828; }
        .tabs { display: flex; gap: 4px; margin-bottom: 24px; border-bottom: 1px solid #e0e0e0; }
        .tab { padding: 12px 24px; background: none; border: none; cursor: pointer; font-size: 14px; color: #666; border-bottom: 2px solid transparent; }
        .tab.active { color: #0066cc; border-bottom-color: #0066cc; }
        .content-grid { display: grid; grid-template-columns: 2fr 1fr; gap: 24px; }
        .posts-section { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
        .section-title { font-size: 18px; font-weight: 600; }
        .post-card { border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; margin-bottom: 12px; }
        .post-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
        .post-platform { display: flex; align-items: center; gap: 8px; }
        .platform-icon { width: 24px; height: 24px; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 12px; color: white; }
        .platform-twitter { background: #1da1f2; }
        .platform-facebook { background: #1877f2; }
        .platform-instagram { background: linear-gradient(45deg, #f09433, #e6683c, #dc2743, #cc2366, #bc1888); }
        .platform-linkedin { background: #0a66c2; }
        .post-status { padding: 4px 10px; border-radius: 20px; font-size: 11px; font-weight: 500; }
        .status-published { background: #e8f5e9; color: #2e7d32; }
        .status-scheduled { background: #fff3e0; color: #ef6c00; }
        .status-draft { background: #f5f5f5; color: #666; }
        .post-content { font-size: 14px; color: #333; line-height: 1.5; margin-bottom: 12px; }
        .post-stats { display: flex; gap: 16px; font-size: 13px; color: #666; }
        .post-stat { display: flex; align-items: center; gap: 4px; }
        .sidebar { display: flex; flex-direction: column; gap: 24px; }
        .sidebar-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .account-item { display: flex; align-items: center; gap: 12px; padding: 12px 0; border-bottom: 1px solid #f0f0f0; }
        .account-item:last-child { border-bottom: none; }
        .account-avatar { width: 40px; height: 40px; border-radius: 50%; background: #e0e0e0; }
        .account-info { flex: 1; }
        .account-name { font-weight: 500; font-size: 14px; }
        .account-handle { font-size: 12px; color: #666; }
        .account-status { width: 8px; height: 8px; border-radius: 50%; }
        .account-status.connected { background: #4caf50; }
        .account-status.disconnected { background: #f44336; }
        .empty-state { text-align: center; padding: 40px; color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Social Media Manager</h1>
            <button class="btn btn-primary" onclick="createPost()">Create Post</button>
        </div>
        <div class="stats-row">
            <div class="stat-card">
                <div class="stat-value" id="totalFollowers">0</div>
                <div class="stat-label">Total Followers</div>
                <div class="stat-change positive">+2.4% this week</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="totalEngagement">0</div>
                <div class="stat-label">Engagement Rate</div>
                <div class="stat-change positive">+0.8% this week</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="postsThisWeek">0</div>
                <div class="stat-label">Posts This Week</div>
                <div class="stat-change">3 scheduled</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="totalReach">0</div>
                <div class="stat-label">Total Reach</div>
                <div class="stat-change positive">+12% this week</div>
            </div>
        </div>
        <div class="tabs">
            <button class="tab active" data-view="posts">Posts</button>
            <button class="tab" data-view="scheduled">Scheduled</button>
            <button class="tab" data-view="drafts">Drafts</button>
            <button class="tab" data-view="analytics">Analytics</button>
        </div>
        <div class="content-grid">
            <div class="posts-section">
                <div class="section-header">
                    <h2 class="section-title">Recent Posts</h2>
                    <select style="padding: 8px 12px; border: 1px solid #ddd; border-radius: 6px;">
                        <option>All Platforms</option>
                        <option>Twitter</option>
                        <option>Facebook</option>
                        <option>Instagram</option>
                        <option>LinkedIn</option>
                    </select>
                </div>
                <div id="postsList">
                    <div class="empty-state">No posts yet. Create your first post to get started.</div>
                </div>
            </div>
            <div class="sidebar">
                <div class="sidebar-card">
                    <div class="section-header">
                        <h3 class="section-title">Connected Accounts</h3>
                        <button class="btn" style="padding: 6px 12px; font-size: 12px;" onclick="connectAccount()">+ Add</button>
                    </div>
                    <div id="accountsList">
                        <div class="empty-state" style="padding: 20px;">No accounts connected</div>
                    </div>
                </div>
                <div class="sidebar-card">
                    <h3 class="section-title" style="margin-bottom: 16px;">Quick Actions</h3>
                    <button class="btn" style="width: 100%; margin-bottom: 8px; background: #f5f5f5; color: #333;" onclick="schedulePost()">Schedule Post</button>
                    <button class="btn" style="width: 100%; margin-bottom: 8px; background: #f5f5f5; color: #333;" onclick="viewCalendar()">Content Calendar</button>
                    <button class="btn" style="width: 100%; background: #f5f5f5; color: #333;" onclick="viewAnalytics()">View Analytics</button>
                </div>
            </div>
        </div>
    </div>
    <script>
        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                loadPosts(tab.dataset.view);
            });
        });

        async function loadPosts(view = 'posts') {
            try {
                const response = await fetch('/api/social/posts');
                const posts = await response.json();
                renderPosts(posts);
            } catch (e) {
                console.error('Failed to load posts:', e);
            }
        }

        async function loadAccounts() {
            try {
                const response = await fetch('/api/social/accounts');
                const accounts = await response.json();
                renderAccounts(accounts);
            } catch (e) {
                console.error('Failed to load accounts:', e);
            }
        }

        function renderPosts(posts) {
            const list = document.getElementById('postsList');
            if (!posts || posts.length === 0) {
                list.innerHTML = '<div class="empty-state">No posts yet. Create your first post to get started.</div>';
                return;
            }
            list.innerHTML = posts.map(p => `
                <div class="post-card">
                    <div class="post-header">
                        <div class="post-platform">
                            <div class="platform-icon platform-${p.platform || 'twitter'}">${getPlatformIcon(p.platform)}</div>
                            <span>${p.platform || 'Twitter'}</span>
                        </div>
                        <span class="post-status status-${p.status || 'draft'}">${p.status || 'Draft'}</span>
                    </div>
                    <div class="post-content">${p.content}</div>
                    <div class="post-stats">
                        <span class="post-stat">❤️ ${p.likes || 0}</span>
                        <span class="post-stat">💬 ${p.comments || 0}</span>
                        <span class="post-stat">🔄 ${p.shares || 0}</span>
                        <span class="post-stat">👁️ ${p.impressions || 0}</span>
                    </div>
                </div>
            `).join('');
        }

        function renderAccounts(accounts) {
            const list = document.getElementById('accountsList');
            if (!accounts || accounts.length === 0) {
                list.innerHTML = '<div class="empty-state" style="padding: 20px;">No accounts connected</div>';
                return;
            }
            list.innerHTML = accounts.map(a => `
                <div class="account-item">
                    <div class="account-avatar" style="background: ${getPlatformColor(a.platform)};"></div>
                    <div class="account-info">
                        <div class="account-name">${a.name}</div>
                        <div class="account-handle">@${a.handle}</div>
                    </div>
                    <div class="account-status ${a.connected ? 'connected' : 'disconnected'}"></div>
                </div>
            `).join('');
        }

        function getPlatformIcon(platform) {
            const icons = { twitter: 'X', facebook: 'f', instagram: '📷', linkedin: 'in' };
            return icons[platform] || 'X';
        }

        function getPlatformColor(platform) {
            const colors = { twitter: '#1da1f2', facebook: '#1877f2', instagram: '#e4405f', linkedin: '#0a66c2' };
            return colors[platform] || '#666';
        }

        function createPost() { window.location = '/suite/social/compose'; }
        function schedulePost() { window.location = '/suite/social/compose?schedule=true'; }
        function viewCalendar() { window.location = '/suite/social/calendar'; }
        function viewAnalytics() { window.location = '/suite/social/analytics'; }
        function connectAccount() { window.location = '/suite/social/accounts'; }

        loadPosts();
        loadAccounts();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_social_compose_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Compose Post</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 900px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .compose-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group textarea { width: 100%; padding: 16px; border: 1px solid #ddd; border-radius: 8px; font-size: 16px; min-height: 150px; resize: vertical; }
        .char-count { text-align: right; font-size: 12px; color: #666; margin-top: 4px; }
        .platforms { display: flex; gap: 12px; flex-wrap: wrap; }
        .platform-btn { padding: 10px 20px; border: 2px solid #ddd; border-radius: 8px; background: white; cursor: pointer; display: flex; align-items: center; gap: 8px; }
        .platform-btn.selected { border-color: #0066cc; background: #e8f4ff; }
        .media-upload { border: 2px dashed #ddd; border-radius: 8px; padding: 32px; text-align: center; cursor: pointer; }
        .media-upload:hover { border-color: #0066cc; background: #f8fafc; }
        .schedule-section { background: #f9f9f9; border-radius: 8px; padding: 20px; margin-bottom: 20px; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-secondary { background: #f5f5f5; color: #333; }
        .form-actions { display: flex; gap: 12px; justify-content: flex-end; }
        .preview-section { margin-top: 24px; padding-top: 24px; border-top: 1px solid #e0e0e0; }
        .preview-title { font-weight: 600; margin-bottom: 12px; }
        .preview-card { border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; max-width: 400px; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/social" class="back-link">← Back to Social</a>
        <div class="compose-card">
            <h1>Compose Post</h1>
            <form id="composeForm">
                <div class="form-group">
                    <label>Select Platforms</label>
                    <div class="platforms">
                        <button type="button" class="platform-btn" data-platform="twitter" onclick="togglePlatform(this)">🐦 Twitter</button>
                        <button type="button" class="platform-btn" data-platform="facebook" onclick="togglePlatform(this)">📘 Facebook</button>
                        <button type="button" class="platform-btn" data-platform="instagram" onclick="togglePlatform(this)">📷 Instagram</button>
                        <button type="button" class="platform-btn" data-platform="linkedin" onclick="togglePlatform(this)">💼 LinkedIn</button>
                    </div>
                </div>
                <div class="form-group">
                    <label>Post Content</label>
                    <textarea id="content" placeholder="What's on your mind?" oninput="updateCharCount()"></textarea>
                    <div class="char-count"><span id="charCount">0</span>/280</div>
                </div>
                <div class="form-group">
                    <label>Media</label>
                    <div class="media-upload" onclick="document.getElementById('mediaInput').click()">
                        <p>📎 Click to upload images or videos</p>
                        <p style="font-size: 12px; color: #999; margin-top: 8px;">Supports JPG, PNG, GIF, MP4</p>
                        <input type="file" id="mediaInput" accept="image/*,video/*" multiple style="display: none;">
                    </div>
                </div>
                <div class="schedule-section">
                    <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
                        <input type="checkbox" id="scheduleToggle" onchange="toggleSchedule()">
                        <span>Schedule for later</span>
                    </label>
                    <div id="scheduleOptions" style="display: none; margin-top: 16px;">
                        <input type="datetime-local" id="scheduleTime" style="padding: 10px; border: 1px solid #ddd; border-radius: 6px;">
                    </div>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="saveDraft()">Save Draft</button>
                    <button type="submit" class="btn btn-primary" id="submitBtn">Post Now</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        let selectedPlatforms = [];

        function togglePlatform(btn) {
            btn.classList.toggle('selected');
            const platform = btn.dataset.platform;
            if (btn.classList.contains('selected')) {
                selectedPlatforms.push(platform);
            } else {
                selectedPlatforms = selectedPlatforms.filter(p => p !== platform);
            }
        }

        function updateCharCount() {
            const content = document.getElementById('content').value;
            document.getElementById('charCount').textContent = content.length;
        }

        function toggleSchedule() {
            const options = document.getElementById('scheduleOptions');
            const submitBtn = document.getElementById('submitBtn');
            if (document.getElementById('scheduleToggle').checked) {
                options.style.display = 'block';
                submitBtn.textContent = 'Schedule Post';
            } else {
                options.style.display = 'none';
                submitBtn.textContent = 'Post Now';
            }
        }

        function saveDraft() {
            const data = {
                content: document.getElementById('content').value,
                platforms: selectedPlatforms,
                status: 'draft'
            };
            fetch('/api/social/posts', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data)
            }).then(() => window.location = '/suite/social');
        }

        document.getElementById('composeForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            if (selectedPlatforms.length === 0) {
                alert('Please select at least one platform');
                return;
            }

            const data = {
                content: document.getElementById('content').value,
                platforms: selectedPlatforms,
                status: document.getElementById('scheduleToggle').checked ? 'scheduled' : 'published',
                scheduled_at: document.getElementById('scheduleToggle').checked ? document.getElementById('scheduleTime').value : null
            };

            try {
                const response = await fetch('/api/social/posts', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });
                if (response.ok) {
                    window.location = '/suite/social';
                } else {
                    alert('Failed to create post');
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

pub async fn handle_social_post_page(
    State(_state): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Post Details</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }}
        .container {{ max-width: 900px; margin: 0 auto; padding: 24px; }}
        .back-link {{ color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }}
        .post-card {{ background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); margin-bottom: 24px; }}
        .post-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }}
        .post-content {{ font-size: 18px; line-height: 1.6; margin-bottom: 20px; }}
        .post-meta {{ color: #666; font-size: 14px; }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }}
        .stat-card {{ background: white; border-radius: 8px; padding: 20px; text-align: center; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }}
        .stat-value {{ font-size: 28px; font-weight: 600; color: #0066cc; }}
        .stat-label {{ font-size: 13px; color: #666; margin-top: 4px; }}
        .btn {{ padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }}
        .btn-danger {{ background: #ffebee; color: #c62828; }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/social" class="back-link">← Back to Social</a>
        <div class="post-card">
            <div class="post-header">
                <h1 id="postPlatform">Loading...</h1>
                <button class="btn btn-danger" onclick="deletePost()">Delete Post</button>
            </div>
            <div class="post-content" id="postContent"></div>
            <div class="post-meta" id="postMeta"></div>
        </div>
        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-value" id="likesCount">0</div>
                <div class="stat-label">Likes</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="commentsCount">0</div>
                <div class="stat-label">Comments</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="sharesCount">0</div>
                <div class="stat-label">Shares</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="impressionsCount">0</div>
                <div class="stat-label">Impressions</div>
            </div>
        </div>
    </div>
    <script>
        const postId = '{post_id}';

        async function loadPost() {{
            try {{
                const response = await fetch(`/api/social/posts/${{postId}}`);
                const post = await response.json();
                if (post) {{
                    document.getElementById('postPlatform').textContent = post.platform || 'Post';
                    document.getElementById('postContent').textContent = post.content;
                    document.getElementById('postMeta').textContent = `Posted on ${{new Date(post.created_at).toLocaleString()}}`;
                    document.getElementById('likesCount').textContent = post.likes || 0;
                    document.getElementById('commentsCount').textContent = post.comments || 0;
                    document.getElementById('sharesCount').textContent = post.shares || 0;
                    document.getElementById('impressionsCount').textContent = post.impressions || 0;
                }}
            }} catch (e) {{
                console.error('Failed to load post:', e);
            }}
        }}

        async function deletePost() {{
            if (!confirm('Are you sure you want to delete this post?')) return;
            try {{
                await fetch(`/api/social/posts/${{postId}}`, {{ method: 'DELETE' }});
                window.location = '/suite/social';
            }} catch (e) {{
                alert('Failed to delete post');
            }}
        }}

        loadPost();
    </script>
</body>
</html>"#
    );
    Html(html)
}

pub fn configure_social_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/social", get(handle_social_list_page))
        .route("/suite/social/compose", get(handle_social_compose_page))
        .route("/suite/social/:id", get(handle_social_post_page))
}
