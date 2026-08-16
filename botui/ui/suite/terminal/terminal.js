(function() {
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

    function getWsUrl(sessionId) {
        const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
        return proto + '//' + location.host + '/api/terminal/ws?id=' + encodeURIComponent(sessionId);
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

        function connect() {
            statusText.textContent = 'Connecting...';
            const token = localStorage.getItem('gb-access-token') || sessionStorage.getItem('gb-access-token');
            // The backend requires a session created via /api/terminal/create
            // before the WS upgrade (it looks up the PTY by ?id=).
            fetch('/api/terminal/create', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': 'Bearer ' + (token || ''),
                },
                body: JSON.stringify({}),
            })
                .then(function (r) { return r.json(); })
                .then(function (info) {
                    if (!info || !info.id) throw new Error('no session id');
                    openSocket(info.id);
                })
                .catch(function (e) {
                    statusText.textContent = 'Failed to start terminal';
                    wsStatus.textContent = 'WS: Error';
                    terminal.write('\r\n\x1b[31m[Failed to create terminal session: ' + e.message + ']\x1b[0m\r\n');
                    reconnectTimer = setTimeout(connect, 3000);
                });
        }

        function openSocket(sessionId) {
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
        }
    };

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
