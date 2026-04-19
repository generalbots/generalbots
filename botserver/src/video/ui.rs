use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

pub async fn handle_video_list_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Video Library</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
        .header h1 { font-size: 28px; color: #1a1a1a; }
        .btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .video-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 24px; }
        .video-card { background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .video-thumbnail { width: 100%; aspect-ratio: 16/9; background: #1a1a1a; position: relative; }
        .video-thumbnail img { width: 100%; height: 100%; object-fit: cover; }
        .video-duration { position: absolute; bottom: 8px; right: 8px; background: rgba(0,0,0,0.8); color: white; padding: 2px 6px; border-radius: 4px; font-size: 12px; }
        .video-info { padding: 16px; }
        .video-title { font-size: 16px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
        .video-meta { font-size: 13px; color: #666; }
        .filters { display: flex; gap: 12px; margin-bottom: 24px; }
        .filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
        .search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; }
        .empty-state { text-align: center; padding: 80px 24px; color: #666; }
        .empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Video Library</h1>
            <button class="btn btn-primary" onclick="uploadVideo()">Upload Video</button>
        </div>
        <div class="filters">
            <input type="text" class="search-box" placeholder="Search videos..." id="searchInput">
            <select class="filter-select" id="categoryFilter">
                <option value="">All Categories</option>
                <option value="training">Training</option>
                <option value="marketing">Marketing</option>
                <option value="product">Product</option>
                <option value="support">Support</option>
            </select>
            <select class="filter-select" id="sortBy">
                <option value="newest">Newest First</option>
                <option value="oldest">Oldest First</option>
                <option value="popular">Most Viewed</option>
            </select>
        </div>
        <div class="video-grid" id="videoGrid">
            <div class="empty-state">
                <h3>No videos yet</h3>
                <p>Upload your first video to get started</p>
            </div>
        </div>
    </div>
    <script>
        async function loadVideos() {
            try {
                const response = await fetch('/api/video/projects');
                const data = await response.json();
                renderVideos(data.projects || []);
            } catch (e) {
                console.error('Failed to load videos:', e);
            }
        }
        function renderVideos(projects) {
            const grid = document.getElementById('videoGrid');
            if (!projects || projects.length === 0) {
                grid.innerHTML = '<div class="empty-state"><h3>No videos yet</h3><p>Upload your first video to get started</p></div>';
                return;
            }
            grid.innerHTML = projects.map(p => `
                <div class="video-card" onclick="window.location='/suite/video/${p.id}'">
                    <div class="video-thumbnail">
                        <img src="${p.thumbnail_url || '/assets/video-placeholder.png'}" alt="${p.name}">
                        <span class="video-duration">${formatDuration(p.duration_ms / 1000)}</span>
                    </div>
                    <div class="video-info">
                        <div class="video-title">${p.name}</div>
                        <div class="video-meta">${p.status} • ${formatDate(p.created_at)}</div>
                    </div>
                </div>
            `).join('');
        }
        function formatDuration(seconds) {
            if (!seconds) return '0:00';
            const mins = Math.floor(seconds / 60);
            const secs = seconds % 60;
            return `${mins}:${secs.toString().padStart(2, '0')}`;
        }
        function formatDate(dateStr) {
            if (!dateStr) return '';
            return new Date(dateStr).toLocaleDateString();
        }
        function uploadVideo() {
            window.location = '/suite/video/upload';
        }
        loadVideos();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_video_detail_page(
    State(_state): State<Arc<AppState>>,
    Path(video_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Video Player</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0a0a0a; color: white; }}
        .container {{ max-width: 1200px; margin: 0 auto; padding: 24px; }}
        .back-link {{ color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }}
        .video-player {{ width: 100%; aspect-ratio: 16/9; background: #000; border-radius: 12px; overflow: hidden; margin-bottom: 24px; }}
        .video-player video {{ width: 100%; height: 100%; }}
        .video-title {{ font-size: 24px; font-weight: 600; margin-bottom: 12px; }}
        .video-meta {{ color: #999; margin-bottom: 24px; }}
        .video-description {{ line-height: 1.6; color: #ccc; margin-bottom: 24px; }}
        .actions {{ display: flex; gap: 12px; margin-bottom: 24px; }}
        .btn {{ padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; }}
        .btn-outline {{ background: transparent; border: 1px solid #444; color: white; }}
        .btn-outline:hover {{ background: #222; }}
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/video" class="back-link">← Back to Library</a>
        <div class="video-player">
            <video id="videoPlayer" controls></video>
        </div>
        <h1 class="video-title" id="videoTitle">Loading...</h1>
        <div class="video-meta" id="videoMeta"></div>
        <div class="video-description" id="videoDescription"></div>
        <div class="actions">
            <button class="btn btn-outline" onclick="shareVideo()">Share</button>
            <button class="btn btn-outline" onclick="downloadVideo()">Download</button>
        </div>
    </div>
    <script>
        const videoId = '{video_id}';
        async function loadVideo() {{
            try {{
                const response = await fetch('/api/video/projects/' + videoId);
                const project = await response.json();
                if (project) {{
                    document.getElementById('videoTitle').textContent = project.name;
                    document.getElementById('videoMeta').textContent = project.status + ' • ' + new Date(project.created_at).toLocaleDateString();
                    document.getElementById('videoDescription').textContent = project.description || '';
                }}
            }} catch (e) {{
                console.error('Failed to load video:', e);
            }}
        }}
        function shareVideo() {{
            navigator.clipboard.writeText(window.location.href);
            alert('Link copied to clipboard!');
        }}
        function downloadVideo() {{
            window.open('/api/video/projects/' + videoId + '/export', '_blank');
        }}
        loadVideo();
    </script>
</body>
</html>"#);
    Html(html)
}

pub async fn handle_video_upload_page(
    State(_state): State<Arc<AppState>>,
) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Upload Video</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .upload-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .upload-zone { border: 2px dashed #ddd; border-radius: 12px; padding: 48px; text-align: center; margin-bottom: 24px; cursor: pointer; }
        .upload-zone:hover { border-color: #0066cc; background: #f8fafc; }
        .upload-zone.dragover { border-color: #0066cc; background: #e8f4ff; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group input, .form-group textarea, .form-group select { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
        .form-group textarea { min-height: 100px; resize: vertical; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .btn-primary:disabled { background: #ccc; cursor: not-allowed; }
        .progress-bar { height: 8px; background: #eee; border-radius: 4px; overflow: hidden; margin-top: 16px; display: none; }
        .progress-bar-fill { height: 100%; background: #0066cc; width: 0%; transition: width 0.3s; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/video" class="back-link">← Back to Library</a>
        <div class="upload-card">
            <h1>Upload Video</h1>
            <div class="upload-zone" id="uploadZone" onclick="document.getElementById('fileInput').click()">
                <p>Drag and drop a video file here, or click to browse</p>
                <p style="color: #999; margin-top: 8px; font-size: 13px;">Supports MP4, WebM, MOV (max 2GB)</p>
                <input type="file" id="fileInput" accept="video/*" style="display: none;">
            </div>
            <div id="selectedFile" style="display: none; margin-bottom: 24px; padding: 12px; background: #f5f5f5; border-radius: 8px;"></div>
            <form id="uploadForm">
                <div class="form-group">
                    <label>Title</label>
                    <input type="text" id="title" required placeholder="Enter video title">
                </div>
                <div class="form-group">
                    <label>Description</label>
                    <textarea id="description" placeholder="Enter video description"></textarea>
                </div>
                <div class="form-group">
                    <label>Category</label>
                    <select id="category">
                        <option value="">Select category</option>
                        <option value="training">Training</option>
                        <option value="marketing">Marketing</option>
                        <option value="product">Product</option>
                        <option value="support">Support</option>
                    </select>
                </div>
                <button type="submit" class="btn btn-primary" id="submitBtn" disabled>Upload Video</button>
                <div class="progress-bar" id="progressBar">
                    <div class="progress-bar-fill" id="progressFill"></div>
                </div>
            </form>
        </div>
    </div>
    <script>
        let selectedFile = null;
        const uploadZone = document.getElementById('uploadZone');
        const fileInput = document.getElementById('fileInput');
        const submitBtn = document.getElementById('submitBtn');

        uploadZone.addEventListener('dragover', (e) => { e.preventDefault(); uploadZone.classList.add('dragover'); });
        uploadZone.addEventListener('dragleave', () => uploadZone.classList.remove('dragover'));
        uploadZone.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadZone.classList.remove('dragover');
            if (e.dataTransfer.files.length) handleFile(e.dataTransfer.files[0]);
        });
        fileInput.addEventListener('change', (e) => { if (e.target.files.length) handleFile(e.target.files[0]); });

        function handleFile(file) {
            if (!file.type.startsWith('video/')) { alert('Please select a video file'); return; }
            selectedFile = file;
            document.getElementById('selectedFile').style.display = 'block';
            document.getElementById('selectedFile').textContent = `Selected: ${file.name} (${(file.size / 1024 / 1024).toFixed(2)} MB)`;
            submitBtn.disabled = false;
            if (!document.getElementById('title').value) {
                document.getElementById('title').value = file.name.replace(/\.[^/.]+$/, '');
            }
        }

        document.getElementById('uploadForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            if (!selectedFile) return;

            const formData = new FormData();
            formData.append('file', selectedFile);
            formData.append('title', document.getElementById('title').value);
            formData.append('description', document.getElementById('description').value);
            formData.append('category', document.getElementById('category').value);

            document.getElementById('progressBar').style.display = 'block';
            submitBtn.disabled = true;

            try {
                const xhr = new XMLHttpRequest();
                xhr.upload.addEventListener('progress', (e) => {
                    if (e.lengthComputable) {
                        const percent = (e.loaded / e.total) * 100;
                        document.getElementById('progressFill').style.width = percent + '%';
                    }
                });
                xhr.addEventListener('load', () => {
                    if (xhr.status === 200) {
                        window.location = '/suite/video';
                    } else {
                        alert('Upload failed');
                        submitBtn.disabled = false;
                    }
                });
                xhr.open('POST', '/api/video/upload');
                xhr.send(formData);
            } catch (e) {
                alert('Upload failed: ' + e.message);
                submitBtn.disabled = false;
            }
        });
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub fn configure_video_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/video", get(handle_video_list_page))
        .route("/suite/video/upload", get(handle_video_upload_page))
        .route("/suite/video/:id", get(handle_video_detail_page))
}
