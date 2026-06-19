"use strict";

(function () {
    let sessionId = null;
    let ws = null;
    let canvas = null;
    let ctx = null;
    let animId = null;
    let agentMode = false;
    let stepCount = 0;

    const elSession = document.getElementById('session-label');
    const elNew = document.getElementById('btn-new-session');
    const elClose = document.getElementById('btn-close-session');
    const elToolbar = document.getElementById('toolbar');
    const elSidebar = document.getElementById('sidebar');
    const elCanvas = document.getElementById('browser-canvas');
    const elStatus = document.getElementById('viewport-status');
    const elOverlay = document.getElementById('viewport-overlay');
    const elUrl = document.getElementById('nav-url');
    const elNav = document.getElementById('btn-navigate');
    const elScreenshot = document.getElementById('btn-screenshot');
    const elExtract = document.getElementById('btn-extract');
    const elBack = document.getElementById('btn-back');
    const elForward = document.getElementById('btn-forward');
    const elAgentChk = document.getElementById('chk-agent-mode');
    const elAgentLog = document.getElementById('agent-log');
    const elStateUrl = document.getElementById('state-url');
    const elStateTitle = document.getElementById('state-title');
    const elStateLinks = document.getElementById('state-links');
    const elStateFps = document.getElementById('state-fps');

    let fps = 0, frameCount = 0, lastFpsTime = Date.now();
    let currentImageData = null;

    canvas = elCanvas;
    ctx = canvas.getContext('2d');

    elNew.addEventListener('click', createSession);
    elClose.addEventListener('click', closeSession);
    elNav.addEventListener('click', navigate);
    elUrl.addEventListener('keydown', function (e) { if (e.key === 'Enter') navigate(); });
    elScreenshot.addEventListener('click', takeScreenshot);
    elExtract.addEventListener('click', extractPage);
    elBack.addEventListener('click', function () { sendCommand('back'); });
    elForward.addEventListener('click', function () { sendCommand('forward'); });
    elAgentChk.addEventListener('change', function () {
        agentMode = this.checked;
        if (agentMode) {
            addLogEntry('info', 'Agent mode enabled');
        } else {
            addLogEntry('info', 'Agent mode disabled');
        }
    });

    canvas.addEventListener('click', function (e) {
        if (!sessionId) return;
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        const x = Math.round((e.clientX - rect.left) * scaleX);
        const y = Math.round((e.clientY - rect.top) * scaleY);
        clickAt(x, y);
    });

    canvas.addEventListener('dblclick', function (e) {
        if (!sessionId) return;
        const rect = canvas.getBoundingClientRect();
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        const x = Math.round((e.clientX - rect.left) * scaleX);
        const y = Math.round((e.clientY - rect.top) * scaleY);
        dblclickAt(x, y);
    });

    async function createSession() {
        try {
            const resp = await fetch('/api/browser/session', { method: 'POST' });
            const data = await resp.json();
            sessionId = data.id;
            elSession.textContent = sessionId.substring(0, 8) + '...';
            elNew.style.display = 'none';
            elClose.style.display = 'inline-block';
            elToolbar.style.display = 'block';
            elSidebar.style.display = 'block';
            elStatus.style.display = 'none';
            canvas.style.display = 'block';
            startStream();
            addLogEntry('info', 'Session created: ' + sessionId);
        } catch (e) {
            addLogEntry('error', 'Failed to create session: ' + e.message);
        }
    }

    async function closeSession() {
        if (!sessionId) return;
        stopStream();
        try {
            await fetch('/api/browser/session/' + sessionId, { method: 'DELETE' });
        } catch (e) { }
        sessionId = null;
        elSession.textContent = 'No session';
        elNew.style.display = 'inline-block';
        elClose.style.display = 'none';
        elToolbar.style.display = 'none';
        elSidebar.style.display = 'none';
        canvas.style.display = 'none';
        elStatus.style.display = 'block';
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        addLogEntry('info', 'Session closed');
    }

    function startStream() {
        if (ws) ws.close();
        ws = new WebSocket('ws://' + location.host + '/api/browser/session/' + sessionId + '/stream');
        ws.binaryType = 'arraybuffer';

        ws.onmessage = function (e) {
            if (e.data instanceof ArrayBuffer) {
                renderFrame(e.data);
            } else {
                try {
                    const msg = JSON.parse(e.data);
                    updateState(msg);
                } catch (err) { }
            }
        };

        ws.onclose = function () {
            if (sessionId) {
                setTimeout(startStream, 2000);
            }
        };

        ws.onerror = function () { };
    }

    function stopStream() {
        if (ws) {
            ws.close();
            ws = null;
        }
        if (animId) {
            cancelAnimationFrame(animId);
            animId = null;
        }
    }

    function renderFrame(data) {
        const blob = new Blob([data], { type: 'image/png' });
        const url = URL.createObjectURL(blob);
        const img = new Image();
        img.onload = function () {
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);
            URL.revokeObjectURL(url);
            frameCount++;
            const now = Date.now();
            if (now - lastFpsTime > 1000) {
                fps = frameCount;
                frameCount = 0;
                lastFpsTime = now;
                elStateFps.textContent = fps;
            }
        };
        img.onerror = function () {
            URL.revokeObjectURL(url);
        };
        img.src = url;
    }

    function updateState(msg) {
        if (msg.url) elStateUrl.textContent = msg.url;
        if (msg.title) elStateTitle.textContent = msg.title;
        if (msg.links !== undefined) elStateLinks.textContent = msg.links;
    }

    async function navigate() {
        const url = elUrl.value.trim();
        if (!url || !sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/navigate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ url: url })
            });
            const data = await resp.json();
            if (data.url) elStateUrl.textContent = data.url;
            addLogEntry('nav', 'Navigated to ' + url);
        } catch (e) {
            addLogEntry('error', 'Navigation failed: ' + e.message);
        }
    }

    async function clickAt(x, y) {
        if (!sessionId) return;
        try {
            await fetch('/api/browser/session/' + sessionId + '/click', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ x: x, y: y })
            });
            addLogEntry('action', 'Click at (' + x + ', ' + y + ')');
        } catch (e) { }
    }

    async function dblclickAt(x, y) {
        if (!sessionId) return;
        try {
            await fetch('/api/browser/session/' + sessionId + '/click', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ x: x, y: y, click_count: 2 })
            });
            addLogEntry('action', 'Double-click at (' + x + ', ' + y + ')');
        } catch (e) { }
    }

    async function takeScreenshot() {
        if (!sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/screenshot');
            const data = await resp.json();
            if (data.image_base64) {
                const img = new Image();
                img.onload = function () {
                    canvas.width = img.width;
                    canvas.height = img.height;
                    ctx.drawImage(img, 0, 0);
                };
                img.src = 'data:image/png;base64,' + data.image_base64;
            }
            addLogEntry('info', 'Screenshot captured');
        } catch (e) { }
    }

    async function extractPage() {
        if (!sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/extract');
            const data = await resp.json();
            elStateTitle.textContent = data.title || '-';
            elStateLinks.textContent = (data.links || []).length;
            addLogEntry('info', 'Page extracted: ' + (data.title || 'unknown'));
        } catch (e) { }
    }

    async function sendCommand(cmd) {
        if (!sessionId) return;
        if (cmd === 'back') {
            try {
                await fetch('/api/browser/session/' + sessionId + '/execute', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ script: 'window.history.back()' })
                });
                addLogEntry('nav', 'Navigated back');
            } catch (e) { }
        } else if (cmd === 'forward') {
            try {
                await fetch('/api/browser/session/' + sessionId + '/execute', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ script: 'window.history.forward()' })
                });
                addLogEntry('nav', 'Navigated forward');
            } catch (e) { }
        }
    }

    function addLogEntry(type, msg) {
        if (!elAgentLog) return;
        const div = document.createElement('div');
        div.className = 'log-entry';
        if (type === 'error') {
            div.style.color = '#e74c3c';
        } else if (type === 'action' || type === 'nav') {
            div.style.color = '#4caf50';
        }
        stepCount++;
        div.innerHTML = '<span class="step-num">[' + stepCount + ']</span> ' +
            '<span class="step-' + (type === 'error' ? 'reason' : 'action') + '">' + msg + '</span>';
        elAgentLog.appendChild(div);
        elAgentLog.scrollTop = elAgentLog.scrollHeight;

        if (elAgentLog.children.length > 200) {
            elAgentLog.removeChild(elAgentLog.firstChild);
        }
    }
})();
