"use strict";

(function () {
    let sessionId = null, ws = null, canvas = null, ctx = null, animId = null;
    let agentMode = false, stepCount = 0, fps = 0, frameCount = 0, lastFpsTime = Date.now();

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
    const elChromeStatus = document.getElementById('browser-status');

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
        addLogEntry('info', agentMode ? 'Agent mode enabled' : 'Agent mode disabled');
    });
    canvas.addEventListener('click', function (e) {
        if (!sessionId) return;
        const p = toCanvasPoint(e);
        clickAt(p.x, p.y);
    });
    canvas.addEventListener('dblclick', function (e) {
        if (!sessionId) return;
        const p = toCanvasPoint(e);
        dblclickAt(p.x, p.y);
    });

    function toCanvasPoint(e) {
        const rect = canvas.getBoundingClientRect();
        return {
            x: Math.round((e.clientX - rect.left) * (canvas.width / rect.width)),
            y: Math.round((e.clientY - rect.top) * (canvas.height / rect.height))
        };
    }

    async function createSession() {
        const api = window.GBDesktopVMTabs;
        if (!api) { addLogEntry('error', 'VM tab strip is unavailable'); return; }
        try {
            const resp = await fetch(api.path(), {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ headless: true, vm: api.vm(), conversation_id: api.conversation(), vm_controlled: true, title: 'New Tab' })
            });
            const data = await resp.json();
            if (!resp.ok || !data.id) throw new Error(data.error || ('HTTP ' + resp.status));
            attachSession(data.id, (data.tab && data.tab.title) || 'New Tab');
            refreshVmTabs();
            addLogEntry('info', 'VM tab opened: ' + data.id);
        } catch (e) {
            addLogEntry('error', 'Failed to create session: ' + errMsg(e));
        }
    }

    async function closeSession() {
        if (!sessionId) return;
        const closing = sessionId;
        stopStream();
        try {
            const api = window.GBDesktopVMTabs;
            const resp = api ? await fetch(api.path('/' + encodeURIComponent(closing)), { method: 'DELETE' }) : { ok: false };
            if (!resp.ok) {
                const fallback = await fetch('/api/browser/session/' + encodeURIComponent(closing), { method: 'DELETE' });
                if (!fallback.ok) throw new Error('Server close failed (HTTP ' + fallback.status + ')');
            }
        } catch (e) {
            const msg = errMsg(e);
            addLogEntry('error', 'Failed to close session on server: ' + msg);
            showChromeStatus('Tab closed locally; server close failed: ' + msg, 'error', true);
            console.warn('[browser] closeSession persistence failure; degraded to local close', { sessionId: closing, error: e });
        }
        clearSessionUi();
        addLogEntry('info', 'Session closed');
    }

    function clearSessionUi() {
        sessionId = null;
        stopStream();
        elSession.textContent = 'No session';
        elNew.style.display = 'inline-block';
        elClose.style.display = 'none';
        elToolbar.style.display = 'none';
        elSidebar.style.display = 'none';
        canvas.style.display = 'none';
        elStatus.style.display = 'block';
        clearViewportError();
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        refreshVmTabs();
    }

    function attachSession(id, title) {
        stopStream();
        sessionId = id;
        elSession.textContent = (title || id).substring(0, 24);
        elNew.style.display = 'none';
        elClose.style.display = 'inline-block';
        elToolbar.style.display = 'block';
        elSidebar.style.display = 'block';
        elStatus.style.display = 'none';
        canvas.style.display = 'block';
        clearViewportError();
        startStream();
        refreshVmTabs();
    }

    function refreshVmTabs() { if (window.GBDesktopVMTabs) window.GBDesktopVMTabs.refresh(); }

    function startStream() {
        if (ws) ws.close();
        const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
        ws = new WebSocket(proto + '//' + location.host + '/api/browser/session/' + sessionId + '/stream');
        ws.binaryType = 'arraybuffer';
        ws.onmessage = function (e) {
            if (e.data instanceof ArrayBuffer) { renderFrame(e.data); return; }
            try { updateState(JSON.parse(e.data)); }
            catch (err) { console.debug('[browser] ignoring non-JSON stream message', { sessionId: sessionId, error: err }); }
        };
        ws.onclose = function () { if (sessionId) setTimeout(startStream, 2000); };
        ws.onerror = function (e) { console.debug('[browser] stream socket error', { sessionId: sessionId, error: e }); };
    }

    function stopStream() {
        if (ws) { ws.close(); ws = null; }
        if (animId) { cancelAnimationFrame(animId); animId = null; }
    }

    function renderFrame(data) {
        const blob = new Blob([data], { type: 'image/png' });
        const url = URL.createObjectURL(blob);
        const img = new Image();
        img.onload = function () {
            canvas.width = img.width; canvas.height = img.height; ctx.drawImage(img, 0, 0);
            URL.revokeObjectURL(url);
            frameCount++;
            const now = Date.now();
            if (now - lastFpsTime > 1000) { fps = frameCount; frameCount = 0; lastFpsTime = now; elStateFps.textContent = fps; }
        };
        img.onerror = function () { URL.revokeObjectURL(url); };
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
        clearViewportError();
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/navigate', {
                method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ url: url })
            });
            let data = {};
            try { data = await resp.json(); } catch (_parseErr) { console.debug('[browser] navigate response was not JSON', { sessionId: sessionId, status: resp.status }); }
            if (!resp.ok) throw new Error((data && data.error) || ('HTTP ' + resp.status));
            if (data.url) elStateUrl.textContent = data.url;
            addLogEntry('nav', 'Navigated to ' + (data.url || url));
            refreshVmTabs();
        } catch (e) {
            reportFailure('Navigation failed', e, { url: url }, { viewport: { title: 'Navigation failed', target: url } });
        }
    }

    async function clickAt(x, y) {
        if (!sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/click', {
                method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ x: x, y: y })
            });
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            addLogEntry('action', 'Click at (' + x + ', ' + y + ')');
        } catch (e) { reportFailure('Click failed', e, { x: x, y: y }); }
    }

    async function dblclickAt(x, y) {
        if (!sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/click', {
                method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ x: x, y: y, click_count: 2 })
            });
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            addLogEntry('action', 'Double-click at (' + x + ', ' + y + ')');
        } catch (e) { reportFailure('Double-click failed', e, { x: x, y: y }); }
    }

    async function takeScreenshot() {
        if (!sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/screenshot');
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            const data = await resp.json();
            if (data.image_base64) {
                const img = new Image();
                img.onload = function () { canvas.width = img.width; canvas.height = img.height; ctx.drawImage(img, 0, 0); };
                img.src = 'data:image/png;base64,' + data.image_base64;
            }
            addLogEntry('info', 'Screenshot captured');
        } catch (e) { reportFailure('Screenshot failed', e); }
    }

    async function extractPage() {
        if (!sessionId) return;
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/extract');
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            const data = await resp.json();
            elStateTitle.textContent = data.title || '-';
            elStateLinks.textContent = (data.links || []).length;
            addLogEntry('info', 'Page extracted: ' + (data.title || 'unknown'));
        } catch (e) { reportFailure('Page extract failed', e); }
    }

    async function sendCommand(cmd) {
        if (!sessionId) return;
        if (cmd !== 'back' && cmd !== 'forward') return;
        clearViewportError();
        const script = cmd === 'back' ? 'window.history.back()' : 'window.history.forward()';
        const label = cmd === 'back' ? 'Back navigation failed' : 'Forward navigation failed';
        const okMsg = cmd === 'back' ? 'Navigated back' : 'Navigated forward';
        try {
            const resp = await fetch('/api/browser/session/' + sessionId + '/execute', {
                method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ script: script })
            });
            if (!resp.ok) throw new Error('HTTP ' + resp.status);
            addLogEntry('nav', okMsg);
        } catch (e) { reportFailure(label, e, { command: cmd }, { viewport: { title: 'Navigation failed', target: script } }); }
    }

    function errMsg(e) { return e && e.message ? e.message : String(e); }
    function reportFailure(label, e, details, opts) {
        const msg = errMsg(e);
        addLogEntry('error', label + ': ' + msg);
        showChromeStatus(label + ': ' + msg, 'error');
        if (opts && opts.viewport) showViewportError(opts.viewport.title, msg, opts.viewport.target);
        console.warn('[browser] ' + label, Object.assign({ sessionId: sessionId, error: e }, details || {}));
    }

    function addLogEntry(type, msg) {
        if (!elAgentLog) return;
        const div = document.createElement('div');
        div.className = 'log-entry';
        div.style.color = type === 'error' ? '#e74c3c' : (type === 'action' || type === 'nav') ? '#4caf50' : '';
        stepCount++;
        div.innerHTML = '<span class="step-num">[' + stepCount + ']</span> <span class="step-' + (type === 'error' ? 'reason' : 'action') + '">' + escapeHtml(String(msg == null ? '' : msg)) + '</span>';
        elAgentLog.appendChild(div);
        elAgentLog.scrollTop = elAgentLog.scrollHeight;
        if (elAgentLog.children.length > 200) elAgentLog.removeChild(elAgentLog.firstChild);
    }

    function escapeHtml(text) { return String(text).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;'); }

    let chromeStatusTimer = null, persistenceErrorShown = false;
    function showChromeStatus(message, kind, persistOnce) {
        if (!elChromeStatus) return;
        if (persistOnce && persistenceErrorShown) return;
        if (persistOnce) persistenceErrorShown = true;
        if (chromeStatusTimer) { clearTimeout(chromeStatusTimer); chromeStatusTimer = null; }
        elChromeStatus.textContent = message;
        elChromeStatus.style.display = 'block';
        elChromeStatus.style.padding = '0.4rem 1.5rem';
        elChromeStatus.style.fontSize = '0.8rem';
        elChromeStatus.style.borderBottom = '1px solid var(--border-color, #2a2a4a)';
        elChromeStatus.style.background = kind === 'error' ? '#3b1d1d' : 'var(--bg-card, #1e1e3a)';
        elChromeStatus.style.color = kind === 'error' ? '#ffb4b4' : 'var(--text-primary, #e0e0e0)';
        if (!persistOnce) {
            chromeStatusTimer = setTimeout(function () {
                if (elChromeStatus) { elChromeStatus.style.display = 'none'; elChromeStatus.textContent = ''; }
                chromeStatusTimer = null;
            }, 6000);
        }
    }

    function showViewportError(title, detail, target) {
        if (!elOverlay) return;
        elOverlay.innerHTML =
            '<div style="font-size:1.1rem;font-weight:600;margin-bottom:0.5rem">' + escapeHtml(title || 'Navigation failed') + '</div>' +
            '<div style="font-size:0.85rem;color:#ffb4b4;margin-bottom:0.5rem">' + escapeHtml(detail || 'The page could not be loaded.') + '</div>' +
            (target ? '<div style="font-size:0.75rem;color:#a0a0b0;word-break:break-all">URL: ' + escapeHtml(target) + '</div>' : '');
        elOverlay.style.display = 'flex';
        elOverlay.style.flexDirection = 'column';
        elOverlay.style.alignItems = 'center';
        elOverlay.style.justifyContent = 'center';
        elOverlay.style.textAlign = 'center';
        elOverlay.style.padding = '1.5rem';
        elOverlay.style.background = 'rgba(13, 13, 26, 0.94)';
        elOverlay.style.color = '#e0e0e0';
        elOverlay.style.pointerEvents = 'none';
    }

    function clearViewportError() {
        if (!elOverlay) return;
        elOverlay.innerHTML = '';
        elOverlay.style.display = 'none';
        elOverlay.style.background = '';
        elOverlay.style.pointerEvents = 'none';
    }

    if (window.GBDesktopVMTabs) {
        window.GBDesktopVMTabs.init({
            onAttach: attachSession,
            getSessionId: function () { return sessionId; },
            onClosed: function (id) { if (sessionId === id) clearSessionUi(); }
        });
    }
})();
