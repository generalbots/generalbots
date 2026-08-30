(function() {
    'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("terminal");

    const MAX_TABS = 10;
    const SCROLLBACK = 10000;
    let tabs = [];
    let activeTab = 0;

    const termBody = document.getElementById('term-body');
    const termTabs = document.getElementById('term-tabs');
    const statusText = document.getElementById('term-status-text');
    const dimensions = document.getElementById('term-dimensions');
    const wsStatus = document.getElementById('term-ws-status');

    function getWsUrl(sessionId) {
        const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
        return proto + '//' + location.host + '/api/terminal/ws?id=' + encodeURIComponent(sessionId);
    }

    /* ---- VM picker: exec into a project VM (Incus container) ---- */
    // Deep-linked from the Vibe ribbon → resolve THAT project's VM.
    // Opened standalone → list every VM across the user's vibe projects.
    const vmSelect = document.getElementById('termVmSelect');
    let vmOptionsLoaded = false;

    function apiGet(path) {
        const token =
            (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) ||
            localStorage.getItem('token') ||
            localStorage.getItem('id_token') ||
            localStorage.getItem('gb-access-token') ||
            sessionStorage.getItem('gb-access-token') ||
            '';
        return fetch(path, { headers: { 'Authorization': 'Bearer ' + token } })
            .then(function (r) { return r.json().catch(function () { return { success: false }; }); });
    }

    function projectFromContext() {
        const params = window.__gbAppParams__ || {};
        const m = window.location.search.match(/[?&]project=([^&]+)/);
        return (params.project || (m && decodeURIComponent(m[1])) || '').toString();
    }

    function preferredVm(vms) {
        if (!Array.isArray(vms) || !vms.length) return null;
        return vms.find(function (v) {
            return String(v.env || '').indexOf('development') !== -1 && String(v.status || '').indexOf('run') !== -1;
        }) || vms.find(function (v) {
            return String(v.status || '').indexOf('run') !== -1;
        }) || vms[0];
    }

    function fillVmSelect(options, preferred) {
        if (!vmSelect) return;
        vmSelect.innerHTML = '';
        // SECURITY: the backend refuses host shells (403) — only project VMs
        // are attachable, so the picker offers nothing else. An empty list
        // disables the selector and the terminal explains how to provision a VM.
        let preselected = false;
        options.forEach(function (o) {
            const opt = document.createElement('option');
            opt.value = o.container || '';
            opt.textContent = o.label;
            if (preferred && o.container === preferred) {
                opt.selected = true;
                preselected = true;
            }
            vmSelect.appendChild(opt);
        });
        vmSelect.disabled = options.length === 0;
        if (options.length && !preselected) {
            vmSelect.selectedIndex = 0;
        }
        vmOptionsLoaded = true;
        // A deep-linked project VM should be used by the (already created)
        // first tab — reconnect it into the container.
        if (preferred && window.GBTerminal) {
            setTimeout(function () {
                if (window.GBTerminal && window.GBTerminal.onVmChange) window.GBTerminal.onVmChange();
            }, 250);
        }
    }

    function loadVmOptions() {
        const projectId = projectFromContext();
        if (!projectId) {
            // Standalone: list VMs across all of the user's projects.
            apiGet('/api/vibe/projects').then(function (data) {
                const projects = (data && (data.projects || (data.success && data.data && data.data.projects))) || [];
                const jobs = projects.slice(0, 10).map(function (p) {
                    const pid = p.project_id || p.id;
                    return apiGet('/api/vibe/projects/' + encodeURIComponent(pid) + '/vms').then(function (vd) {
                        const vms = (vd && (vd.vms || (vd.success && vd.data && vd.data.vms))) || [];
                        return vms.map(function (v) {
                            return {
                                container: v.container_name || '',
                                label: (p.name || pid).substring(0, 24) + ' · ' + (v.env || 'vm') + (v.status ? ' (' + v.status + ')' : ''),
                            };
                        });
                    }).catch(function () { return []; });
                });
                return Promise.all(jobs);
            }).then(function (groups) {
                const flat = [];
                groups.forEach(function (g) { if (Array.isArray(g)) flat.push.apply(flat, g); });
                const list = flat.filter(function (o) { return o.container; });
                // Prefer a running VM (label carries the status), else the first.
                const running = list.find(function (o) { return /run/i.test(o.label || ''); });
                const pref = running || list[0] || null;
                fillVmSelect(list, pref ? pref.container : null);
            }).catch(function () { vmOptionsLoaded = true; });
        } else {
            // From the Vibe ribbon: resolve that project's VM and preselect it.
            apiGet('/api/vibe/projects/' + encodeURIComponent(projectId) + '/vms').then(function (vd) {
                const vms = (vd && (vd.vms || (vd.success && vd.data && vd.data.vms))) || [];
                const pref = preferredVm(vms);
                fillVmSelect(vms.map(function (v) {
                    return {
                        container: v.container_name || '',
                        label: (v.env || 'vm') + (v.status ? ' (' + v.status + ')' : ''),
                    };
                }).filter(function (o) { return o.container; }), pref && pref.container_name);
            }).catch(function () { vmOptionsLoaded = true; /* picker stays on default */ });
        }
    }

    function getSelectedContainer() {
        if (!vmOptionsLoaded || !vmSelect) return '';
        return (vmSelect.value || '').trim();
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
        const webLinksAddon = new WebLinksAddon.WebLinksAddon();
        terminal.loadAddon(fitAddon);
        terminal.loadAddon(webLinksAddon);
        terminal.open(pane);

        let ws = null;
        let reconnectTimer = null;
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
            if (!vmOptionsLoaded) {
                // The VM picker loads async; never race it with a host shell.
                statusText.textContent = 'Resolving VM…';
                const wait = setInterval(function () {
                    if (vmOptionsLoaded) {
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
            statusText.textContent = 'Connecting...';
            const token =
                (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) ||
                localStorage.getItem('token') ||
                localStorage.getItem('id_token') ||
                localStorage.getItem('gb-access-token') ||
                sessionStorage.getItem('gb-access-token');
            const container = getSelectedContainer();
            // SECURITY: host shells are refused server-side (403). With no VM
            // available, explain how to provision one instead of attempting a
            // bare botserver shell or a workspace-cwd fallback.
            if (!container) {
                statusText.textContent = 'No VM';
                wsStatus.textContent = 'WS: —';
                terminal.write('\r\n\x1b[31m[No project VM available for your account.]\x1b[0m\r\n');
                terminal.write('\x1b[33m[Open Vibe, start a project (Play) to provision its VM, then reopen the terminal.]\x1b[0m\r\n');
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
                .then(function (r) { return r.json(); })
                .then(function (info) {
                    connecting = false;
                    if (!info || !info.id) throw new Error('no session id');
                    openSocket(info.id);
                })
                .catch(function (e) {
                    connecting = false;
                    statusText.textContent = 'Failed to start terminal';
                    wsStatus.textContent = 'WS: Error';
                    terminal.write('\r\n\x1b[31m[Failed to create terminal session: ' + e.message + ']\x1b[0m\r\n');
                    reconnectTimer = setTimeout(connect, 3000);
                });
        }

        function openSocket(sessionId) {
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
            ws = new WebSocket(getWsUrl(sessionId));
            ws.onopen = function() {
                wsStatus.textContent = 'WS: Connected';
                statusText.textContent = 'Connected';
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
                    statusText.textContent = 'Connecting...';
                    // New shell, new prompt: start it on a fresh line so the
                    // previous `root@name>` prompt is never buried mid-line.
                    terminal.write('\r\n\x1b[90m[switching VM — new shell]\x1b[0m\r\n');
                    connect();
                    return;
                }
                wsStatus.textContent = 'WS: Disconnected';
                statusText.textContent = 'Disconnected';
                terminal.write('\r\n\x1b[33m[Connection closed. Reconnecting in 3s...]\x1b[0m\r\n');
                reconnectTimer = setTimeout(connect, 3000);
            };
            ws.onerror = function() {
                wsStatus.textContent = 'WS: Error';
            };
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
    }

    window.GBTerminal = {
        addTab: function() {
            if (tabs.length >= MAX_TABS) return;
            var idx = tabs.length;
            var tab = createTerminal(idx);
            tabs.push(tab);
            switchTab(idx);
            updateTabs();
        },
        closeTab: function(index) {
            if (tabs.length <= 1) return;
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
    };

    if (vmSelect) {
        vmSelect.addEventListener('change', function () { window.GBTerminal.onVmChange(); });
    }
    loadVmOptions();

    var firstTab = createTerminal(0);
    tabs.push(firstTab);
    updateTabs();

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
})();
