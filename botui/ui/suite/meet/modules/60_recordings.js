/* Meet — LiveKit recording orchestration and recordings panel. */
(function () {
    "use strict";

    // ---------------------------------------------------------------------
    // Meeting recording (LiveKit egress orchestration)
    // ---------------------------------------------------------------------

    var recording = {
        active: false,
        recordingId: null,
        startedAt: null,
        timerInterval: null
    };

    function apiFetch(url, options) {
        options = options || {};
        options.headers = Object.assign({ 'Content-Type': 'application/json' }, options.headers || {});
        var token =
            (typeof window.getAccessToken === 'function' && window.getAccessToken()) ||
            sessionStorage.getItem('gb_access_token') ||
            localStorage.getItem('gb_access_token');
        if (token) {
            options.headers.Authorization = 'Bearer ' + token;
        }
        return fetch(url, options).then(function (resp) {
            if (!resp.ok) {
                return resp.json().catch(function () { return {}; }).then(function (body) {
                    throw new Error((body && body.error) || ('HTTP ' + resp.status));
                });
            }
            return resp.json();
        });
    }

    function currentUserId() {
        var el = document.getElementById('current-user-id');
        if (el && el.value) return el.value;
        try {
            var auth = JSON.parse(sessionStorage.getItem('gb_auth_user') || localStorage.getItem('gb_auth_user') || 'null');
            if (auth && auth.id) return auth.id;
        } catch (e) { /* ignore */ }
        return null;
    }

    function formatElapsed(ms) {
        var total = Math.floor(ms / 1000);
        var h = Math.floor(total / 3600);
        var m = Math.floor((total % 3600) / 60);
        var s = total % 60;
        var pad = function (n) { return n < 10 ? '0' + n : '' + n; };
        return (h > 0 ? pad(h) + ':' : '') + pad(m) + ':' + pad(s);
    }

    function setRecordingUI(active) {
        recording.active = active;
        var btn = document.getElementById('record-btn');
        var indicator = document.getElementById('recording-indicator');
        var label = document.getElementById('record-label');
        if (btn) btn.classList.toggle('recording', active);
        if (indicator) indicator.classList.toggle('hidden', !active);
        if (label) label.textContent = active ? 'Stop' : 'Record';
    }

    function startRecordingTimer() {
        recording.startedAt = Date.now();
        if (recording.timerInterval) clearInterval(recording.timerInterval);
        recording.timerInterval = setInterval(function () {
            var el = document.getElementById('recording-timer');
            if (el && recording.startedAt) {
                el.textContent = formatElapsed(Date.now() - recording.startedAt);
            }
        }, 1000);
    }

    function stopRecordingTimer() {
        if (recording.timerInterval) {
            clearInterval(recording.timerInterval);
            recording.timerInterval = null;
        }
    }

    window.toggleRecording = async function () {
        var state = window._meetState;
        var roomId = state && state.roomId;
        if (!roomId) {
            window.showNotification('Join a meeting to record');
            return;
        }
        var userId = currentUserId();

        if (!recording.active) {
            try {
                var result = await apiFetch('/api/meet/rooms/' + roomId + '/recording/start', {
                    method: 'POST',
                    body: JSON.stringify({
                        user_id: userId,
                        webinar_id: roomId,
                        enable_transcription: true,
                        transcription_language: 'en-US'
                    })
                });
                recording.recordingId = result.id;
                setRecordingUI(true);
                startRecordingTimer();
                window.showNotification('Recording started');
            } catch (err) {
                console.error('Failed to start recording:', err);
                window.showNotification('Failed to start recording: ' + err.message);
            }
        } else {
            try {
                await apiFetch('/api/meet/rooms/' + roomId + '/recording/stop', {
                    method: 'POST',
                    body: JSON.stringify({
                        user_id: userId,
                        recording_id: recording.recordingId
                    })
                });
                setRecordingUI(false);
                stopRecordingTimer();
                recording.recordingId = null;
                window.showNotification('Recording stopped');
                loadRecordings();
            } catch (err) {
                console.error('Failed to stop recording:', err);
                window.showNotification('Failed to stop recording: ' + err.message);
            }
        }
    };

    window.openRecordingsPanel = function () {
        var panel = document.getElementById('recordings-panel');
        if (panel) panel.classList.remove('hidden');
        loadRecordings();
    };

    window.closeRecordingsPanel = function () {
        var panel = document.getElementById('recordings-panel');
        if (panel) panel.classList.add('hidden');
    };

    window.loadRecordings = function () {
        var state = window._meetState;
        var roomId = state && state.roomId;
        var list = document.getElementById('recordings-list');
        if (!roomId || !list) return;

        apiFetch('/api/meet/rooms/' + roomId + '/recordings')
            .then(function (items) {
                if (!items || !items.length) {
                    list.innerHTML =
                        '<div class="recordings-empty">No recordings yet</div>';
                    return;
                }
                list.innerHTML = items.map(function (r) {
                    var ready = r.status === 'ready';
                    var statusText = {
                        recording: 'Recording',
                        processing: 'Processing',
                        ready: 'Ready',
                        failed: 'Failed'
                    }[r.status] || r.status;
                    var duration =
                        typeof r.duration_seconds === 'number'
                            ? formatElapsed(r.duration_seconds * 1000)
                            : '—';
                    return (
                        '<div class="recording-item">' +
                        '<div class="recording-item-info">' +
                        '<div class="recording-item-title">Meeting recording · ' + statusText + '</div>' +
                        '<div class="recording-item-meta">' + duration + ' · ' + new Date(r.started_at).toLocaleString() + '</div>' +
                        '</div>' +
                        '<div class="recording-item-actions">' +
                        (ready
                            ? '<button title="Play" onclick="playRecording(\'' + r.id + '\')">▶</button>' +
                              '<button title="Download" onclick="downloadRecording(\'' + r.id + '\')">⬇</button>'
                            : '') +
                        '<button class="recording-delete" title="Delete" onclick="deleteRecording(\'' + r.id + '\')">🗑</button>' +
                        '</div>' +
                        '</div>'
                    );
                }).join('');
            })
            .catch(function (err) {
                console.error('Failed to load recordings:', err);
                list.innerHTML =
                    '<div class="recordings-empty">Failed to load recordings</div>';
            });
    };

    window.playRecording = function (recordingId) {
        var url = '/api/meet/recordings/' + encodeURIComponent(String(recordingId)) + '/file';
        var win = window.open('', '_blank');
        if (!win) return;
        // Build the player with DOM APIs: the URL never becomes markup (#2340).
        win.document.title = 'Recording';
        var video = win.document.createElement('video');
        video.src = url;
        video.controls = true;
        video.autoplay = true;
        video.style.width = '100%';
        video.style.height = '100vh';
        win.document.body.appendChild(video);
    };

    window.downloadRecording = function (recordingId) {
        window.location.href = '/api/meet/recordings/' + recordingId + '/file';
    };

    window.deleteRecording = function (recordingId) {
        if (!(window.GBModal||{confirm:confirm}).confirm('Delete this recording?')) return;
        apiFetch('/api/meet/recordings/' + recordingId, { method: 'DELETE' })
            .then(function () {
                window.showNotification('Recording deleted');
                loadRecordings();
            })
            .catch(function (err) {
                console.error('Failed to delete recording:', err);
                window.showNotification('Failed to delete recording: ' + err.message);
            });
    };

    // Clean up the timer when leaving the room.
    var _origLeaveRoom = window.leaveRoom;
    window.leaveRoom = function () {
        stopRecordingTimer();
        recording.active = false;
        recording.recordingId = null;
        if (_origLeaveRoom) return _origLeaveRoom.apply(this, arguments);
    };
})();
