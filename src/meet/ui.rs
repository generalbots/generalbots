use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

pub async fn handle_meet_list_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Meetings</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; }
        .header h1 { font-size: 28px; color: #1a1a1a; }
        .btn { padding: 10px 20px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .btn-success { background: #2e7d32; color: white; }
        .btn-success:hover { background: #1b5e20; }
        .stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; margin-bottom: 24px; }
        .stat-card { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        .stat-value { font-size: 28px; font-weight: 600; color: #1a1a1a; }
        .stat-label { font-size: 13px; color: #666; margin-top: 4px; }
        .tabs { display: flex; gap: 4px; margin-bottom: 24px; border-bottom: 1px solid #e0e0e0; }
        .tab { padding: 12px 24px; background: none; border: none; cursor: pointer; font-size: 14px; color: #666; border-bottom: 2px solid transparent; }
        .tab.active { color: #0066cc; border-bottom-color: #0066cc; }
        .meeting-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 24px; }
        .meeting-card { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; }
        .meeting-card:hover { transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.12); }
        .meeting-card.live { border-left: 4px solid #2e7d32; }
        .meeting-status { display: inline-block; padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: 500; margin-bottom: 12px; }
        .status-live { background: #e8f5e9; color: #2e7d32; }
        .status-scheduled { background: #e3f2fd; color: #1565c0; }
        .status-ended { background: #f5f5f5; color: #666; }
        .meeting-title { font-size: 18px; font-weight: 600; color: #1a1a1a; margin-bottom: 8px; }
        .meeting-time { font-size: 14px; color: #666; margin-bottom: 12px; display: flex; align-items: center; gap: 8px; }
        .meeting-participants { display: flex; align-items: center; gap: 8px; }
        .participant-avatars { display: flex; }
        .participant-avatar { width: 32px; height: 32px; border-radius: 50%; background: #e0e0e0; border: 2px solid white; margin-left: -8px; display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 500; color: #666; }
        .participant-avatar:first-child { margin-left: 0; }
        .participant-count { font-size: 13px; color: #666; }
        .meeting-actions { display: flex; gap: 8px; margin-top: 16px; }
        .meeting-actions .btn { padding: 8px 16px; font-size: 13px; }
        .filters { display: flex; gap: 12px; margin-bottom: 24px; }
        .search-box { flex: 1; padding: 10px 16px; border: 1px solid #ddd; border-radius: 8px; }
        .filter-select { padding: 8px 16px; border: 1px solid #ddd; border-radius: 8px; background: white; }
        .empty-state { text-align: center; padding: 80px 24px; color: #666; }
        .empty-state h3 { margin-bottom: 8px; color: #1a1a1a; }
        .quick-join { background: white; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); margin-bottom: 24px; display: flex; gap: 12px; align-items: center; }
        .quick-join input { flex: 1; padding: 12px 16px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Meetings</h1>
            <div style="display: flex; gap: 12px;">
                <button class="btn btn-primary" onclick="scheduleMeeting()">📅 Schedule Meeting</button>
                <button class="btn btn-success" onclick="startInstantMeeting()">🎥 Start Instant Meeting</button>
            </div>
        </div>
        <div class="quick-join">
            <span style="font-weight: 500;">Join a meeting:</span>
            <input type="text" id="meetingCode" placeholder="Enter meeting code or link">
            <button class="btn btn-primary" onclick="joinMeeting()">Join</button>
        </div>
        <div class="stats-row">
            <div class="stat-card">
                <div class="stat-value" id="liveMeetings">0</div>
                <div class="stat-label">Live Now</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="todayMeetings">0</div>
                <div class="stat-label">Today's Meetings</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="weekMeetings">0</div>
                <div class="stat-label">This Week</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="totalHours">0h</div>
                <div class="stat-label">Meeting Hours (Month)</div>
            </div>
        </div>
        <div class="tabs">
            <button class="tab active" data-view="upcoming">Upcoming</button>
            <button class="tab" data-view="live">Live Now</button>
            <button class="tab" data-view="past">Past Meetings</button>
            <button class="tab" data-view="recordings">Recordings</button>
        </div>
        <div class="filters">
            <input type="text" class="search-box" placeholder="Search meetings..." id="searchInput" oninput="filterMeetings()">
            <select class="filter-select" id="typeFilter" onchange="filterMeetings()">
                <option value="">All Types</option>
                <option value="instant">Instant</option>
                <option value="scheduled">Scheduled</option>
                <option value="recurring">Recurring</option>
            </select>
            <select class="filter-select" id="sortBy" onchange="filterMeetings()">
                <option value="date">Sort by Date</option>
                <option value="name">Sort by Name</option>
                <option value="participants">Sort by Participants</option>
            </select>
        </div>
        <div class="meeting-grid" id="meetingGrid">
            <div class="empty-state">
                <h3>No meetings scheduled</h3>
                <p>Schedule a meeting or start an instant meeting to get started</p>
            </div>
        </div>
    </div>
    <script>
        let meetings = [];
        let currentView = 'upcoming';

        document.querySelectorAll('.tab').forEach(tab => {
            tab.addEventListener('click', () => {
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                tab.classList.add('active');
                currentView = tab.dataset.view;
                filterMeetings();
            });
        });

        async function loadMeetings() {
            try {
                const response = await fetch('/api/meet/rooms');
                meetings = await response.json();
                filterMeetings();
                updateStats();
            } catch (e) {
                console.error('Failed to load meetings:', e);
            }
        }

        function filterMeetings() {
            let filtered = meetings;
            const query = document.getElementById('searchInput').value.toLowerCase();
            const type = document.getElementById('typeFilter').value;

            if (currentView === 'live') {
                filtered = filtered.filter(m => m.status === 'live' || m.is_active);
            } else if (currentView === 'upcoming') {
                filtered = filtered.filter(m => m.status === 'scheduled' || (!m.ended_at && !m.is_active));
            } else if (currentView === 'past') {
                filtered = filtered.filter(m => m.status === 'ended' || m.ended_at);
            }

            if (query) {
                filtered = filtered.filter(m =>
                    (m.name && m.name.toLowerCase().includes(query)) ||
                    (m.topic && m.topic.toLowerCase().includes(query))
                );
            }

            if (type) {
                filtered = filtered.filter(m => m.meeting_type === type);
            }

            renderMeetings(filtered);
        }

        function renderMeetings(meetings) {
            const grid = document.getElementById('meetingGrid');
            if (!meetings || meetings.length === 0) {
                grid.innerHTML = '<div class="empty-state"><h3>No meetings found</h3><p>Try a different filter or create a new meeting</p></div>';
                return;
            }

            grid.innerHTML = meetings.map(m => {
                const isLive = m.status === 'live' || m.is_active;
                return `
                    <div class="meeting-card ${isLive ? 'live' : ''}" onclick="openMeeting('${m.id}')">
                        <span class="meeting-status status-${isLive ? 'live' : (m.ended_at ? 'ended' : 'scheduled')}">
                            ${isLive ? '🔴 Live' : (m.ended_at ? 'Ended' : 'Scheduled')}
                        </span>
                        <div class="meeting-title">${m.name || m.topic || 'Untitled Meeting'}</div>
                        <div class="meeting-time">
                            📅 ${formatDateTime(m.scheduled_at || m.created_at)}
                            ${m.duration ? ` • ${m.duration} min` : ''}
                        </div>
                        <div class="meeting-participants">
                            <div class="participant-avatars">
                                ${(m.participants || []).slice(0, 3).map((p, i) => `
                                    <div class="participant-avatar" style="background: ${getAvatarColor(i)}">${(p.name || 'U')[0]}</div>
                                `).join('')}
                            </div>
                            <span class="participant-count">${m.participant_count || (m.participants || []).length} participants</span>
                        </div>
                        <div class="meeting-actions">
                            ${isLive ?
                                `<button class="btn btn-success" onclick="event.stopPropagation(); joinRoom('${m.id}')">Join Now</button>` :
                                `<button class="btn btn-primary" onclick="event.stopPropagation(); startMeeting('${m.id}')">Start</button>`
                            }
                            <button class="btn" style="background: #f5f5f5;" onclick="event.stopPropagation(); copyLink('${m.id}')">Copy Link</button>
                        </div>
                    </div>
                `;
            }).join('');
        }

        function formatDateTime(dateStr) {
            if (!dateStr) return 'Not scheduled';
            const date = new Date(dateStr);
            const now = new Date();
            const tomorrow = new Date(now);
            tomorrow.setDate(tomorrow.getDate() + 1);

            if (date.toDateString() === now.toDateString()) {
                return `Today at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
            }
            if (date.toDateString() === tomorrow.toDateString()) {
                return `Tomorrow at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
            }
            return date.toLocaleDateString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
        }

        function getAvatarColor(index) {
            const colors = ['#e3f2fd', '#f3e5f5', '#e8f5e9', '#fff3e0', '#fce4ec'];
            return colors[index % colors.length];
        }

        function updateStats() {
            const live = meetings.filter(m => m.status === 'live' || m.is_active).length;
            document.getElementById('liveMeetings').textContent = live;
            document.getElementById('todayMeetings').textContent = meetings.length;
        }

        function scheduleMeeting() {
            window.location = '/suite/meet/schedule';
        }

        async function startInstantMeeting() {
            try {
                const response = await fetch('/api/meet/create', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name: 'Instant Meeting', created_by: 'user' })
                });
                const meeting = await response.json();
                window.location = `/suite/meet/room/${meeting.id}`;
            } catch (e) {
                alert('Failed to create meeting');
            }
        }

        function joinMeeting() {
            const code = document.getElementById('meetingCode').value.trim();
            if (!code) {
                alert('Please enter a meeting code');
                return;
            }
            window.location = `/suite/meet/join/${code}`;
        }

        function openMeeting(id) {
            window.location = `/suite/meet/${id}`;
        }

        function joinRoom(id) {
            window.location = `/suite/meet/room/${id}`;
        }

        function startMeeting(id) {
            window.location = `/suite/meet/room/${id}`;
        }

        function copyLink(id) {
            const link = `${window.location.origin}/suite/meet/join/${id}`;
            navigator.clipboard.writeText(link);
            alert('Meeting link copied to clipboard!');
        }

        loadMeetings();
    </script>
</body>
</html>"#;
    Html(html.to_string())
}

pub async fn handle_meet_room_page(
    State(_state): State<Arc<AppState>>,
    Path(room_id): Path<Uuid>,
) -> Html<String> {
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Meeting Room</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #1a1a1a; color: white; height: 100vh; overflow: hidden; }}
        .meeting-container {{ display: flex; height: 100vh; }}
        .video-area {{ flex: 1; display: flex; flex-direction: column; }}
        .video-grid {{ flex: 1; display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px; padding: 16px; }}
        .video-tile {{ background: #2d2d2d; border-radius: 12px; position: relative; aspect-ratio: 16/9; display: flex; align-items: center; justify-content: center; overflow: hidden; }}
        .video-tile video {{ width: 100%; height: 100%; object-fit: cover; }}
        .video-tile .participant-name {{ position: absolute; bottom: 12px; left: 12px; background: rgba(0,0,0,0.6); padding: 4px 12px; border-radius: 4px; font-size: 13px; }}
        .video-tile .muted-indicator {{ position: absolute; bottom: 12px; right: 12px; background: rgba(0,0,0,0.6); padding: 4px 8px; border-radius: 4px; font-size: 12px; }}
        .video-tile.screen-share {{ grid-column: span 2; grid-row: span 2; }}
        .controls-bar {{ background: #2d2d2d; padding: 16px; display: flex; justify-content: center; gap: 12px; }}
        .control-btn {{ width: 56px; height: 56px; border-radius: 50%; border: none; cursor: pointer; font-size: 20px; display: flex; align-items: center; justify-content: center; transition: background 0.2s; }}
        .control-btn.active {{ background: #3d3d3d; color: white; }}
        .control-btn.inactive {{ background: #c62828; color: white; }}
        .control-btn.end {{ background: #c62828; color: white; width: auto; border-radius: 28px; padding: 0 24px; font-size: 14px; font-weight: 500; }}
        .control-btn:hover {{ opacity: 0.9; }}
        .sidebar {{ width: 320px; background: #2d2d2d; display: none; flex-direction: column; }}
        .sidebar.open {{ display: flex; }}
        .sidebar-header {{ padding: 16px; border-bottom: 1px solid #3d3d3d; display: flex; justify-content: space-between; align-items: center; }}
        .sidebar-header h3 {{ font-size: 16px; }}
        .sidebar-close {{ background: none; border: none; color: white; font-size: 20px; cursor: pointer; }}
        .sidebar-content {{ flex: 1; overflow-y: auto; padding: 16px; }}
        .participant-item {{ display: flex; align-items: center; gap: 12px; padding: 12px 0; border-bottom: 1px solid #3d3d3d; }}
        .participant-avatar {{ width: 40px; height: 40px; border-radius: 50%; background: #4a4a4a; display: flex; align-items: center; justify-content: center; }}
        .participant-info {{ flex: 1; }}
        .participant-name-list {{ font-weight: 500; }}
        .participant-status {{ font-size: 12px; color: #999; }}
        .chat-messages {{ flex: 1; overflow-y: auto; padding: 16px; }}
        .chat-message {{ margin-bottom: 16px; }}
        .chat-sender {{ font-weight: 500; font-size: 13px; margin-bottom: 4px; }}
        .chat-text {{ font-size: 14px; line-height: 1.4; color: #ccc; }}
        .chat-input {{ display: flex; gap: 8px; padding: 16px; border-top: 1px solid #3d3d3d; }}
        .chat-input input {{ flex: 1; padding: 10px 16px; border: 1px solid #3d3d3d; border-radius: 8px; background: #1a1a1a; color: white; }}
        .chat-input button {{ padding: 10px 20px; background: #0066cc; border: none; border-radius: 8px; color: white; cursor: pointer; }}
        .meeting-info {{ position: absolute; top: 16px; left: 16px; background: rgba(0,0,0,0.6); padding: 8px 16px; border-radius: 8px; font-size: 13px; }}
        .meeting-timer {{ font-weight: 600; }}
    </style>
</head>
<body>
    <div class="meeting-container">
        <div class="video-area">
            <div class="meeting-info">
                <span class="meeting-timer" id="meetingTimer">00:00:00</span>
                <span> • Meeting ID: {room_id}</span>
            </div>
            <div class="video-grid" id="videoGrid">
                <div class="video-tile">
                    <video id="localVideo" autoplay muted playsinline></video>
                    <span class="participant-name">You</span>
                </div>
            </div>
            <div class="controls-bar">
                <button class="control-btn active" id="micBtn" onclick="toggleMic()">🎤</button>
                <button class="control-btn active" id="camBtn" onclick="toggleCam()">📹</button>
                <button class="control-btn active" id="screenBtn" onclick="toggleScreen()">🖥️</button>
                <button class="control-btn active" onclick="toggleChat()">💬</button>
                <button class="control-btn active" onclick="toggleParticipants()">👥</button>
                <button class="control-btn active" onclick="toggleWhiteboard()">📝</button>
                <button class="control-btn active" onclick="toggleRecord()">⏺️</button>
                <button class="control-btn end" onclick="leaveMeeting()">Leave Meeting</button>
            </div>
        </div>
        <div class="sidebar" id="participantsSidebar">
            <div class="sidebar-header">
                <h3>Participants (<span id="participantCount">1</span>)</h3>
                <button class="sidebar-close" onclick="toggleParticipants()">×</button>
            </div>
            <div class="sidebar-content" id="participantsList">
                <div class="participant-item">
                    <div class="participant-avatar">Y</div>
                    <div class="participant-info">
                        <div class="participant-name-list">You (Host)</div>
                        <div class="participant-status">Connected</div>
                    </div>
                </div>
            </div>
        </div>
        <div class="sidebar" id="chatSidebar">
            <div class="sidebar-header">
                <h3>Chat</h3>
                <button class="sidebar-close" onclick="toggleChat()">×</button>
            </div>
            <div class="chat-messages" id="chatMessages">
                <div class="chat-message">
                    <div class="chat-sender">System</div>
                    <div class="chat-text">Meeting started. Share the meeting link to invite others.</div>
                </div>
            </div>
            <div class="chat-input">
                <input type="text" id="chatInput" placeholder="Type a message..." onkeypress="if(event.key==='Enter')sendChat()">
                <button onclick="sendChat()">Send</button>
            </div>
        </div>
    </div>
    <script>
        const roomId = '{room_id}';
        let micEnabled = true;
        let camEnabled = true;
        let screenSharing = false;
        let startTime = new Date();

        async function initMedia() {{
            try {{
                const stream = await navigator.mediaDevices.getUserMedia({{ video: true, audio: true }});
                document.getElementById('localVideo').srcObject = stream;
            }} catch (e) {{
                console.error('Failed to get media:', e);
            }}
        }}

        function toggleMic() {{
            micEnabled = !micEnabled;
            const btn = document.getElementById('micBtn');
            btn.className = `control-btn ${{micEnabled ? 'active' : 'inactive'}}`;
            btn.textContent = micEnabled ? '🎤' : '🔇';
        }}

        function toggleCam() {{
            camEnabled = !camEnabled;
            const btn = document.getElementById('camBtn');
            btn.className = `control-btn ${{camEnabled ? 'active' : 'inactive'}}`;
            btn.textContent = camEnabled ? '📹' : '📷';
        }}

        async function toggleScreen() {{
            const btn = document.getElementById('screenBtn');
            if (!screenSharing) {{
                try {{
                    const stream = await navigator.mediaDevices.getDisplayMedia({{ video: true }});
                    screenSharing = true;
                    btn.className = 'control-btn inactive';
                }} catch (e) {{
                    console.error('Screen share failed:', e);
                }}
            }} else {{
                screenSharing = false;
                btn.className = 'control-btn active';
            }}
        }}

        function toggleChat() {{
            const sidebar = document.getElementById('chatSidebar');
            const participantsSidebar = document.getElementById('participantsSidebar');
            participantsSidebar.classList.remove('open');
            sidebar.classList.toggle('open');
        }}

        function toggleParticipants() {{
            const sidebar = document.getElementById('participantsSidebar');
            const chatSidebar = document.getElementById('chatSidebar');
            chatSidebar.classList.remove('open');
            sidebar.classList.toggle('open');
        }}

        function toggleWhiteboard() {{
            window.open(`/suite/meet/room/${{roomId}}/whiteboard`, '_blank', 'width=1200,height=800');
        }}

        function toggleRecord() {{
            alert('Recording feature coming soon');
        }}

        function sendChat() {{
            const input = document.getElementById('chatInput');
            const message = input.value.trim();
            if (!message) return;

            const messagesEl = document.getElementById('chatMessages');
            messagesEl.innerHTML += `
                <div class="chat-message">
                    <div class="chat-sender">You</div>
                    <div class="chat-text">${{message}}</div>
                </div>
            `;
            messagesEl.scrollTop = messagesEl.scrollHeight;
            input.value = '';
        }}

        function leaveMeeting() {{
            if (confirm('Are you sure you want to leave the meeting?')) {{
                window.location = '/suite/meet';
            }}
        }}

        function updateTimer() {{
            const now = new Date();
            const diff = now - startTime;
            const hours = Math.floor(diff / 3600000).toString().padStart(2, '0');
            const minutes = Math.floor((diff % 3600000) / 60000).toString().padStart(2, '0');
            const seconds = Math.floor((diff % 60000) / 1000).toString().padStart(2, '0');
            document.getElementById('meetingTimer').textContent = `${{hours}}:${{minutes}}:${{seconds}}`;
        }}

        setInterval(updateTimer, 1000);
        initMedia();
    </script>
</body>
</html>"#);
    Html(html)
}

pub async fn handle_meet_schedule_page(State(_state): State<Arc<AppState>>) -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Schedule Meeting</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; }
        .container { max-width: 700px; margin: 0 auto; padding: 24px; }
        .back-link { color: #0066cc; text-decoration: none; display: inline-block; margin-bottom: 16px; }
        .form-card { background: white; border-radius: 12px; padding: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
        h1 { font-size: 24px; margin-bottom: 24px; }
        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 500; margin-bottom: 8px; }
        .form-group input, .form-group textarea, .form-group select { width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 8px; font-size: 14px; }
        .form-group textarea { min-height: 100px; resize: vertical; }
        .form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
        .checkbox-group { display: flex; align-items: center; gap: 8px; }
        .checkbox-group input { width: auto; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; }
        .btn-primary { background: #0066cc; color: white; }
        .btn-primary:hover { background: #0052a3; }
        .btn-secondary { background: #f5f5f5; color: #333; }
        .form-actions { display: flex; gap: 12px; justify-content: flex-end; }
        .settings-section { background: #f9f9f9; border-radius: 8px; padding: 20px; margin-bottom: 20px; }
        .settings-title { font-weight: 600; margin-bottom: 12px; }
    </style>
</head>
<body>
    <div class="container">
        <a href="/suite/meet" class="back-link">← Back to Meetings</a>
        <div class="form-card">
            <h1>Schedule Meeting</h1>
            <form id="meetingForm">
                <div class="form-group">
                    <label>Meeting Title</label>
                    <input type="text" id="name" required placeholder="Enter meeting title">
                </div>
                <div class="form-group">
                    <label>Description (optional)</label>
                    <textarea id="description" placeholder="Add meeting description or agenda"></textarea>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label>Date</label>
                        <input type="date" id="date" required>
                    </div>
                    <div class="form-group">
                        <label>Time</label>
                        <input type="time" id="time" required>
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label>Duration</label>
                        <select id="duration">
                            <option value="15">15 minutes</option>
                            <option value="30">30 minutes</option>
                            <option value="45">45 minutes</option>
                            <option value="60" selected>1 hour</option>
                            <option value="90">1.5 hours</option>
                            <option value="120">2 hours</option>
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Meeting Type</label>
                        <select id="meetingType">
                            <option value="scheduled">One-time Meeting</option>
                            <option value="recurring">Recurring Meeting</option>
                        </select>
                    </div>
                </div>
                <div class="settings-section">
                    <div class="settings-title">Meeting Settings</div>
                    <div class="form-group">
                        <label class="checkbox-group">
                            <input type="checkbox" id="waitingRoom" checked>
                            <span>Enable waiting room</span>
                        </label>
                    </div>
                    <div class="form-group">
                        <label class="checkbox-group">
                            <input type="checkbox" id="muteOnEntry" checked>
                            <span>Mute participants on entry</span>
                        </label>
                    </div>
                    <div class="form-group">
                        <label class="checkbox-group">
                            <input type="checkbox" id="allowRecording">
                            <span>Allow recording</span>
                        </label>
                    </div>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="window.location='/suite/meet'">Cancel</button>
                    <button type="submit" class="btn btn-primary">Schedule Meeting</button>
                </div>
            </form>
        </div>
    </div>
    <script>
        const today = new Date();
        document.getElementById('date').valueAsDate = today;
        document.getElementById('time').value = '09:00';

        document.getElementById('meetingForm').addEventListener('submit', async (e) => {
            e.preventDefault();

            const date = document.getElementById('date').value;
            const time = document.getElementById('time').value;
            const scheduledAt = new Date(`${date}T${time}`).toISOString();

            const data = {
                name: document.getElementById('name').value,
                description: document.getElementById('description').value || null,
                scheduled_at: scheduledAt,
                duration: parseInt(document.getElementById('duration').value),
                meeting_type: document.getElementById('meetingType').value,
                settings: {
                    waiting_room: document.getElementById('waitingRoom').checked,
                    mute_on_entry: document.getElementById('muteOnEntry').checked,
                    allow_recording: document.getElementById('allowRecording').checked
                }
            };

            try {
                const response = await fetch('/api/meet/create', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name: data.title, created_by: 'user', settings: data.settings })
                });

                if (response.ok) {
                    const meeting = await response.json();
                    window.location = `/suite/meet/${meeting.id}`;
                } else {
                    alert('Failed to schedule meeting');
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

pub fn configure_meet_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/suite/meet", get(handle_meet_list_page))
        .route("/suite/meet/schedule", get(handle_meet_schedule_page))
        .route("/suite/meet/room/:id", get(handle_meet_room_page))
}
