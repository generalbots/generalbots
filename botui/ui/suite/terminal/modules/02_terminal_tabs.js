/* Terminal app — module 2/2: panes, tabs and public GBTerminal API (split from terminal/terminal.js for #1731) */

(function (app) {
    'use strict';

    const MAX_TABS = 10;
    const SCROLLBACK = 10000;
    let tabs = [];
    let activeTab = 0;

    const termBody = document.getElementById('term-body');
    const termTabs = document.getElementById('term-tabs');
    const statusText = document.getElementById('term-status-text');
    const dimensions = document.getElementById('term-dimensions');
    const wsStatus = document.getElementById('term-ws-status');

    // #1849 — mirror terminal state into the shared window status bar. The
    // in-app footer remains for standalone/legacy use, but the window footer
    // is the single cross-app status surface.
    function terminalMetrics() {
        return [
            { label: 'Size', value: dimensions.textContent || '--', title: 'Terminal size (columns x rows)' },
            { label: 'WS', value: wsStatus.textContent || 'WS: --', title: 'Terminal WebSocket status' },
            { label: 'Tabs', value: tabs.length, title: 'Open terminal tabs' }
        ];
    }

    function publishTerminalStatus(state, text, extra) {
        if (!window.GBStatus || typeof window.GBStatus.set !== 'function') return;
        var payload = {
            state: state || 'ready',
            text: text || 'Ready',
            metrics: terminalMetrics(),
            busy: state === 'loading' || state === 'busy' || state === 'connecting',
            progress: (state === 'loading' || state === 'connecting')
                ? { indeterminate: true, label: text || 'Connecting...' }
                : null
        };
        if (extra) Object.keys(extra).forEach(function (key) { payload[key] = extra[key]; });
        window.GBStatus.set('terminal', payload);
    }

    function setTermStatus(text, state, extra) {
        statusText.textContent = text;
        statusText.dataset.state = state || 'ready';
        publishTerminalStatus(state || 'ready', text, extra);
    }

    function createTerminal(index) {
        const pane = document.createElement('div');
        pane.className = 'term-pane' + (index === activeTab ? ' active' : '');
        pane.dataset.pane = index;
        termBody.appendChild(pane);

        const terminal = new Terminal({
            theme: {
                background: '#0f172a',
                foreground: '#f8fafc',
                cursor: '#3b82f6',
                cursorAccent: '#0f172a',
                selectionBackground: 'rgba(59,130,246,0.3)',
                black: '#1e293b',
                red: '#ef4444',
                green: '#22c55e',
                yellow: '#eab308',
                blue: '#3b82f6',
                magenta: '#a855f7',
                cyan: '#06b6d4',
                white: '#f8fafc',
                brightBlack: '#64748b',
                brightRed: '#f87171',
                brightGreen: '#4ade80',
                brightYellow: '#facc15',
                brightBlue: '#60a5fa',
                brightMagenta: '#c084fc',
                brightCyan: '#22d3ee',
                brightWhite: '#ffffff'
            },
            fontFamily: '"Fira Code", "Cascadia Code", "JetBrains Mono", monospace',
            fontSize: 14,
            lineHeight: 1.2,
            cursorBlink: true,
            cursorStyle: 'bar',
            scrollback: SCROLLBACK,
            allowProposedApi: true
        });

        const fitAddon = new FitAddon.FitAddon();
        terminal.loadAddon(fitAddon);
        // Monaco's AMD loader can capture UMD define() calls made earlier in
        // the page, so window.WebLinksAddon may be missing. Links are
        // optional — never let them break the terminal.
        try {
            const globalAddons = (typeof WebLinksAddon !== "undefined") ? WebLinksAddon : window.WebLinksAddon;
            if (globalAddons) terminal.loadAddon(new (globalAddons.WebLinksAddon || globalAddons)());
        } catch (e) {
            console.warn("WebLinksAddon unavailable, links disabled", e);
        }
        terminal.open(pane);

        let ws = null;
        let reconnectTimer = null;
        let openTimer = null;
        // When the user switches VM in the picker, closing the old socket is
        // INTENTIONAL — suppress the yellow "Reconnecting" message and just
        // switch straight to the new container (green OK), no 3s delay.
        let switchingVm = false;
        // Guards against two PTYs being spawned at once. Switching the VM
        // used to call connect() BOTH directly (reconnect()) AND from the
        // old socket's onclose — every session prints its own `root@name>`
        // prompt, so each switch stacked another prompt into the same pane
        // ("prompt repeated 3 times" bug). Only ONE session may exist.
        let connecting = false;
        let currentSessionId = null;

        function connect() {
            if (!app._vm.isReady()) {
                // The VM picker loads async; never race it with a host shell.
                setTermStatus('Resolving VM…', 'loading');
                const wait = setInterval(function () {
                    if (app._vm.isReady()) {
                        clearInterval(wait);
                        doConnect();
                    }
                }, 150);
                setTimeout(function () { clearInterval(wait); }, 15000); // safety net
                return;
            }
            doConnect();
        }

        function doConnect() {
            setTermStatus('Connecting...', 'connecting');
            // #1841 — never call an endpoint this build does not ship.
            const unavailable = app._vm.capabilityMessage();
            if (unavailable) {
                wsStatus.textContent = 'WS: —';
                setTermStatus('Unavailable', 'error');
                terminal.write('\r\n\x1b[31m[' + unavailable + ']\x1b[0m\r\n');
                terminal.write('\x1b[33m[Chat and file output keep working; the terminal needs the desktop edition\'s agent VM.]\x1b[0m\r\n');
                return;
            }
            const token =
                (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) ||
                localStorage.getItem('token') ||
                localStorage.getItem('id_token') ||
                localStorage.getItem('gb-access-token') ||
                sessionStorage.getItem('gb-access-token');
            const container = app._vm.getSelectedContainer();
            // SECURITY: host shells are refused server-side (403). With no VM
            // available, explain how to provision one instead of attempting a
            // bare botserver shell or a workspace-cwd fallback.
            if (!container) {
                wsStatus.textContent = 'WS: —';
                setTermStatus('No VM', 'error');
                terminal.write('\r\n\x1b[31m[No workspace VM available for your account.]\x1b[0m\r\n');
                terminal.write('\x1b[33m[Send a message in chat to start your workspace VM, then reopen the terminal.]\x1b[0m\r\n');
                return;
            }
            // The backend requires a session created via /api/terminal/create
            // before the WS upgrade (it looks up the PTY by ?id=).
            createSession({ container: container }, token);
        }

        function createSession(body, token) {
            if (connecting) return; // never spawn a second PTY mid-switch
            connecting = true;
            fetch('/api/terminal/create', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': 'Bearer ' + (token || ''),
                },
                body: JSON.stringify(body),
            })
                .then(function (r) {
                    return r.text().then(function (text) {
                        var data = {};
                        try { data = text ? JSON.parse(text) : {}; } catch (e) { data = { error: text }; }
                        return { ok: r.ok, status: r.status, data: data };
                    });
                })
                .then(function (result) {
                    connecting = false;
                    if (!result.ok || !result.data || !result.data.id) {
                        // Raw payload for support; actionable category for the user.
                        console.error('Terminal create failed:', result.status, result.data);
                        wsStatus.textContent = 'WS: Error';
                        setTermStatus('Failed to start terminal', 'error');
                        terminal.write('\r\n\x1b[31m[' + app._vm.errorMessage(result.data, result.status) + ']\x1b[0m\r\n');
                        return;
                    }
                    openSocket(result.data.id);
                })
                .catch(function (e) {
                    connecting = false;
                    wsStatus.textContent = 'WS: Error';
                    setTermStatus('Failed to start terminal', 'error');
                    console.error('Terminal create request failed:', e);
                    terminal.write('\r\n\x1b[31m[Terminal unavailable: ' + (e && e.message ? e.message : e) + ']\x1b[0m\r\n');
                });
        }

        function openSocket(sessionId) {
            if (openTimer) clearTimeout(openTimer);
            openTimer = null;
            // The previous PTY (old VM) is still alive server-side; retire it
            // so its shell and prompt cannot linger or leak.
            if (currentSessionId && currentSessionId !== sessionId) {
                fetch('/api/terminal/kill', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ id: currentSessionId }),
                }).catch(function () { /* best-effort */ });
            }
            currentSessionId = sessionId;
            ws = new WebSocket(app._vm.getWsUrl(sessionId));
            ws.onopen = function() {
                wsStatus.textContent = 'WS: Connected';
                setTermStatus('Connected', 'connected');
                fitAddon.fit();
                ws.send('resize ' + terminal.cols + ' ' + terminal.rows);
            };
            ws.onmessage = function(event) {
                try {
                    var data = JSON.parse(event.data);
                    if (data.type === 'output') {
                        terminal.write(data.data);
                    }
                } catch (e) {
                    terminal.write(event.data);
                }
            };
            ws.onclose = function() {
                if (switchingVm) {
                    // User-chosen VM switch: reconnect immediately to the new
                    // container, no yellow pending message, no artificial delay.
                    switchingVm = false;
                    wsStatus.textContent = 'WS: Connected';
                    setTermStatus('Connecting...', 'connecting');
                    // New shell, new prompt: start it on a fresh line so the
                    // previous `root@name>` prompt is never buried mid-line.
                    terminal.write('\r\n\x1b[90m[switching VM — new shell]\x1b[0m\r\n');
                    connect();
                    return;
                }
                wsStatus.textContent = 'WS: Disconnected';
                setTermStatus('Disconnected', 'offline');
                terminal.write('\r\n\x1b[33m[Connection closed. Reconnecting in 3s...]\x1b[0m\r\n');
                reconnectTimer = setTimeout(connect, 3000);
            };
            ws.onerror = function() {
                wsStatus.textContent = 'WS: Error';
                setTermStatus('WS: Error', 'error');
            };
            // #1288 — a WS upgrade that never completes (proxy hiccup, hung
            // backend) otherwise leaves the status at "Connecting..."
            // forever. If the socket has not opened within 10s, close it and
            // retry through the normal onclose path.
            openTimer = setTimeout(function () {
                openTimer = null;
                if (ws && ws.readyState !== WebSocket.OPEN) {
                    wsStatus.textContent = 'WS: Timeout';
                    setTermStatus('WS: Timeout', 'error');
                    try { ws.close(); } catch (ignore) { }
                }
            }, 10000);
        }

        terminal.onData(function(data) {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(data);
            }
        });

        terminal.onResize(function(size) {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send('resize ' + size.cols + ' ' + size.rows);
            }
            dimensions.textContent = size.cols + 'x' + size.rows;
            publishTerminalStatus(statusText.dataset.state || 'ready', statusText.textContent || 'Ready');
        });

        var observer = new ResizeObserver(function() {
            if (tabs[index] && tabs[index].visible) {
                fitAddon.fit();
            }
        });
        observer.observe(pane);

        connect();

        return {
            terminal: terminal,
            fitAddon: fitAddon,
            ws: ws,
            pane: pane,
            reconnectTimer: reconnectTimer,
            visible: index === activeTab,
            reconnect: function () {
                // Recreate the PTY in the newly selected container. Closing
                // the old socket lets its onclose handler reconnect — the
                // single path. Calling connect() here AS WELL spawned a
                // second PTY whose prompt stacked on top of the first.
                switchingVm = true;
                if (reconnectTimer) clearTimeout(reconnectTimer);
                if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
                    try { ws.close(); } catch (ignore) { }
                } else {
                    connect();
                }
            },
            cleanup: function() {
                if (reconnectTimer) clearTimeout(reconnectTimer);
                if (ws) ws.close();
                observer.disconnect();
                terminal.dispose();
                pane.remove();
            }
        };
    }

    function switchTab(index) {
        if (index < 0 || index >= tabs.length) return;
        var oldTab = tabs[activeTab];
        if (oldTab) {
            oldTab.visible = false;
            oldTab.pane.classList.remove('active');
        }
        activeTab = index;
        var newTab = tabs[index];
        if (newTab) {
            newTab.visible = true;
            newTab.pane.classList.add('active');
            newTab.fitAddon.fit();
        }
        updateTabs();
    }

    function updateTabs() {
        var html = '';
        for (var i = 0; i < tabs.length; i++) {
            var cls = i === activeTab ? ' active' : '';
            html += '<div class="term-tab' + cls + '" data-tab="' + i + '" onclick="GBTerminal.switchTab(' + i + ')">'
                + '<span class="term-tab-label">Terminal ' + (i + 1) + '</span>'
                + '<span class="term-tab-close" onclick="event.stopPropagation();GBTerminal.closeTab(' + i + ')">&times;</span>'
                + '</div>';
        }
        if (tabs.length < MAX_TABS) {
            html += '<button class="term-tab-add" onclick="GBTerminal.addTab()">+</button>';
        }
        termTabs.innerHTML = html;
        if (statusText.dataset.state) {
            publishTerminalStatus(statusText.dataset.state, statusText.textContent || 'Ready');
        }
    }


    Object.assign(app, {
        addTab: function() {
            if (tabs.length >= MAX_TABS) return;
            var idx = tabs.length;
            var tab = createTerminal(idx);
            tabs.push(tab);
            switchTab(idx);
            updateTabs();
        },
        closeTab: function(index) {
            // #1288 — closing the LAST tab used to be a silent no-op (the
            // `tabs.length <= 1` guard), which read as "the X does nothing".
            // Closing the last tab now replaces it with a fresh session so
            // the ✕ always visibly closes the current shell.
            if (tabs.length <= 1) {
                tabs[0].cleanup();
                var fresh = createTerminal(0);
                tabs[0] = fresh;
                activeTab = 0;
                fresh.visible = true;
                fresh.pane.classList.add('active');
                fresh.fitAddon.fit();
                updateTabs();
                return;
            }
            tabs[index].cleanup();
            tabs.splice(index, 1);
            if (activeTab >= tabs.length) activeTab = tabs.length - 1;
            for (var i = 0; i < tabs.length; i++) {
                tabs[i].pane.dataset.pane = i;
            }
            switchTab(activeTab);
            updateTabs();
        },
        switchTab: switchTab,
        clearCurrent: function() {
            if (tabs[activeTab]) tabs[activeTab].terminal.clear();
        },
        find: function() {
            if (tabs[activeTab]) {
                tabs[activeTab].terminal.focus();
            }
        },
        onVmChange: function() {
            if (tabs[activeTab] && tabs[activeTab].reconnect) tabs[activeTab].reconnect();
        }
    });

    var firstTab = createTerminal(0);
    tabs.push(firstTab);
    updateTabs();

    // Hover help through the shared status contract (#1849).
    if (window.GBStatus && typeof window.GBStatus.registerHelp === 'function') {
        window.GBStatus.registerHelp('#termVmSelect', 'Choose the workspace or project VM for this shell');
        window.GBStatus.registerHelp('#btn-add-tab', 'Open a new terminal tab (Ctrl+T)');
        document.querySelectorAll('.term-action-btn').forEach(function (btn) {
            var label = String(btn.textContent || '').trim();
            if (label) window.GBStatus.registerHelp(btn, label + ' terminal');
        });
    }

    document.addEventListener('keydown', function(e) {
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
            e.preventDefault();
            window.GBTerminal.find();
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 't') {
            e.preventDefault();
            window.GBTerminal.addTab();
        }
        if ((e.ctrlKey || e.metaKey) && e.key === 'w') {
            e.preventDefault();
            window.GBTerminal.closeTab(activeTab);
        }
    });
})(window.GBTerminal);
