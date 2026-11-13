(function() {
    'use strict';

    const API_BASE = '/api/desktop';
    let currentSession = null;
    let ws = null;
    let canvas = null;
    let ctx = null;
    let frameCount = 0;
    let lastFpsTime = Date.now();
    let connections = [];

    const stateEl = document.getElementById('connection-status-text');
    const overlayEl = document.getElementById('connection-overlay');
    const sessionSection = document.getElementById('session-section');
    const connectionsSection = document.getElementById('connections-section');
    const grid = document.getElementById('connections-grid');
    const sessionCount = document.getElementById('session-count');

    function setState(text) {
        if (stateEl) stateEl.textContent = text;
    }

    function showOverlay(show) {
        if (overlayEl) overlayEl.style.display = show ? 'flex' : 'none';
    }

    function updateSessionBadge(count) {
        if (sessionCount) {
            sessionCount.textContent = count;
            sessionCount.style.display = count > 0 ? 'inline' : 'none';
        }
    }

    function maskIp(host) {
        if (!host) return '***';
        const parts = host.split('.');
        if (parts.length === 4) {
            return parts[0] + '.' + parts[1] + '.x.x';
        }
        return host.substring(0, 3) + '***';
    }

    function sanitize(str) {
        const div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }

    async function loadConnections() {
        try {
            const resp = await fetch(API_BASE + '/connections');
            if (!resp.ok) throw new Error('Failed to load connections');
            const body = await resp.json();
            // Handler returns { success, data: [...] } — accept a bare array too.
            connections = Array.isArray(body) ? body : (Array.isArray(body && body.data) ? body.data : []);
            renderConnections();
        } catch (e) {
            console.error('loadConnections:', e);
        }
    }

    function renderConnections() {
        if (!grid) return;
        if (connections.length === 0) {
            grid.innerHTML = '<div class="connection-card empty-state"><p>No saved connections. Create one or use Quick Connect above.</p></div>';
            return;
        }
        grid.innerHTML = connections.map(c => `
            <div class="connection-card" data-id="${sanitize(c.id)}">
                <div class="conn-card-header">
                    <span class="conn-card-name">${sanitize(c.name)}</span>
                    <span class="conn-card-status offline"></span>
                </div>
                <div class="conn-card-details">
                    ${sanitize(c.host)}:${c.port} (${sanitize(c.protocol).toUpperCase()})
                    ${c.last_used_at ? '<br>Last: ' + timeAgo(c.last_used_at) : ''}
                </div>
                <div class="conn-card-actions">
                    <button class="btn-connect" onclick="DesktopVDI.connectSaved('${sanitize(c.id)}')">Connect</button>
                    <button class="btn-remove" onclick="DesktopVDI.removeConnection('${sanitize(c.id)}')">Remove</button>
                </div>
            </div>
        `).join('');
    }

    function timeAgo(isoStr) {
        const diff = Date.now() - new Date(isoStr).getTime();
        const mins = Math.floor(diff / 60000);
        if (mins < 1) return 'just now';
        if (mins < 60) return mins + 'm ago';
        const hrs = Math.floor(mins / 60);
        if (hrs < 24) return hrs + 'h ago';
        return Math.floor(hrs / 24) + 'd ago';
    }

    async function connect(host, port, protocol) {
        if (currentSession) {
            disconnect();
        }

        protocol = protocol || 'vnc';
        const sessionId = crypto.randomUUID();

        showOverlay(true);
        setState('Connecting to ' + maskIp(host) + ':' + port + '...');

        const wsProtocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = wsProtocol + '//' + location.host + API_BASE + '/ws/' + sessionId;

        ws = new WebSocket(wsUrl);
        ws.binaryType = 'arraybuffer';

        ws.onopen = function() {
            ws.send(JSON.stringify({ type: 'connect', host: host, port: parseInt(port), protocol: protocol }));
            setState('Authenticating...');
        };

        ws.onmessage = function(evt) {
            if (typeof evt.data === 'string') {
                handleControlMessage(JSON.parse(evt.data));
            } else {
                handleFrame(evt.data);
            }
        };

        ws.onclose = function() {
            setState('Disconnected');
            showOverlay(false);
            currentSession = null;
            updateSessionBadge(0);
        };

        ws.onerror = function(e) {
            console.error('WebSocket error:', e);
            setState('Connection error');
            showOverlay(false);
        };

        currentSession = { id: sessionId, host: host, port: port, protocol: protocol };

        if (connectionsSection) connectionsSection.style.display = 'none';
        if (sessionSection) sessionSection.style.display = 'flex';
        updateSessionBadge(1);
    }

    function handleControlMessage(msg) {
        switch (msg.type) {
            case 'connected':
                setState('Connected');
                showOverlay(false);
                break;
            case 'auth_required':
                showAuthModal();
                break;
            case 'error':
                setState('Error: ' + (msg.message || 'Unknown'));
                showOverlay(false);
                break;
            case 'frame':
                handleFrame(msg.data);
                break;
        }
    }

    function handleFrame(data) {
        if (!canvas) {
            canvas = document.getElementById('vnc-canvas');
            ctx = canvas ? canvas.getContext('2d') : null;
            if (canvas && !ctx) {
                ctx = canvas.getContext('2d');
            }
        }

        frameCount++;
        const now = Date.now();
        if (now - lastFpsTime >= 1000) {
            const fpsEl = document.getElementById('status-fps');
            if (fpsEl) fpsEl.textContent = 'FPS: ' + frameCount;
            frameCount = 0;
            lastFpsTime = now;
        }

        if (data instanceof ArrayBuffer) {
            const blob = new Blob([data], { type: 'image/png' });
            const img = new Image();
            const url = URL.createObjectURL(blob);
            img.onload = function() {
                if (canvas && ctx) {
                    canvas.width = img.width;
                    canvas.height = img.height;
                    ctx.drawImage(img, 0, 0);
                }
                URL.revokeObjectURL(url);
            };
            img.src = url;
        }
    }

    function showAuthModal() {
        const overlay = document.getElementById('auth-modal-overlay');
        if (overlay) overlay.style.display = 'flex';
    }

    function disconnect() {
        if (ws) {
            ws.close();
            ws = null;
        }
        currentSession = null;
        if (sessionSection) sessionSection.style.display = 'none';
        if (connectionsSection) connectionsSection.style.display = 'block';
        showOverlay(false);
        updateSessionBadge(0);
    }

    async function saveAndConnect() {
        const name = document.getElementById('conn-name').value.trim();
        const host = document.getElementById('conn-host').value.trim();
        const port = parseInt(document.getElementById('conn-port').value) || 5900;
        const protocol = document.getElementById('conn-protocol').value;
        const authType = document.getElementById('conn-auth-type').value;

        if (!host) {
            alert('Host is required');
            return;
        }

        try {
            const resp = await fetch(API_BASE + '/connect', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: name || host, host, port, protocol, auth_type: authType })
            });
            if (resp.ok) {
                closeModal();
                loadConnections();
                connect(host, port, protocol);
            }
        } catch (e) {
            console.error('saveAndConnect:', e);
            connect(host, port, protocol);
        }
    }

    function closeModal() {
        const overlay = document.getElementById('modal-overlay');
        if (overlay) overlay.style.display = 'none';
    }

    async function removeConnection(id) {
        if (!confirm('Remove this connection?')) return;
        try {
            await fetch(API_BASE + '/connections/' + id, { method: 'DELETE' });
            loadConnections();
        } catch (e) {
            console.error('removeConnection:', e);
        }
    }

    function sendAuth() {
        const passwordEl = document.getElementById('auth-password');
        if (!passwordEl) return;
        const password = passwordEl.value;
        if (ws && password) {
            ws.send(JSON.stringify({ type: 'auth', password: password }));
            const authOverlay = document.getElementById('auth-modal-overlay');
            if (authOverlay) authOverlay.style.display = 'none';
            passwordEl.value = '';
        }
    }

    function sendCtrlAltDel() {
        if (ws) {
            ws.send(JSON.stringify({ type: 'key', keys: ['Control_L', 'Alt_L', 'Delete'] }));
        }
    }

    function init() {
        const btnNewConnection = document.getElementById('btn-new-connection');
        if (btnNewConnection) {
            btnNewConnection.addEventListener('click', function() {
                const modalOverlay = document.getElementById('modal-overlay');
                if (modalOverlay) modalOverlay.style.display = 'flex';
            });
        }
        const btnModalClose = document.getElementById('btn-modal-close');
        if (btnModalClose) btnModalClose.addEventListener('click', closeModal);
        const btnModalCancel = document.getElementById('btn-modal-cancel');
        if (btnModalCancel) btnModalCancel.addEventListener('click', closeModal);
        const btnModalSave = document.getElementById('btn-modal-save');
        if (btnModalSave) btnModalSave.addEventListener('click', saveAndConnect);
        const btnQuickConnect = document.getElementById('btn-quick-connect');
        if (btnQuickConnect) {
            btnQuickConnect.addEventListener('click', function() {
                const quickHost = document.getElementById('quick-host');
                const val = quickHost ? quickHost.value.trim() : '';
                if (!val) return;
                const parts = val.split(':');
                connect(parts[0], parseInt(parts[1], 10) || 5900, 'vnc');
            });
        }
        const btnDisconnect = document.getElementById('btn-disconnect');
        if (btnDisconnect) btnDisconnect.addEventListener('click', disconnect);
        const btnCtrlAltDel = document.getElementById('btn-ctrl-alt-del');
        if (btnCtrlAltDel) btnCtrlAltDel.addEventListener('click', sendCtrlAltDel);
        const btnAuthSubmit = document.getElementById('btn-auth-submit');
        if (btnAuthSubmit) btnAuthSubmit.addEventListener('click', sendAuth);

        const quickHostInput = document.getElementById('quick-host');
        if (quickHostInput) {
            quickHostInput.addEventListener('keydown', function(e) {
                if (e.key === 'Enter') {
                    const btn = document.getElementById('btn-quick-connect');
                    if (btn) btn.click();
                }
            });
        }

        if (grid || connectionsSection) {
            loadConnections();
        }
    }

    window.DesktopVDI = {
        connectSaved: async function(id) {
            const c = connections.find(x => x.id === id);
            if (c) connect(c.host, c.port, c.protocol);
        },
        removeConnection: removeConnection,
        init: init
    };

    if (document.readyState === 'loading') {
        (function(){ var __cb = init; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
    } else {
        init();
    }
})();
