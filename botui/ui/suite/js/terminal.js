const botCoderTerminal = {
    term: null,
    ws: null,
    sessionId: null,
    reconnectAttempts: 0,
    maxReconnectAttempts: 5,

    init: function() {
        const container = document.getElementById('xtermContainer');
        if (!container) return;
        if (!window.Terminal) {
            console.error('xterm.js not loaded. Cannot init terminal.');
            container.innerHTML = '<div class="botcoder-error">Terminal library not found. Run "npm install xterm" to install it.</div>';
            return;
        }

        this.term = new Terminal({
            theme: {
                background: '#0f172a',
                foreground: '#f8fafc',
                cursor: '#3b82f6',
                selectionBackground: 'rgba(59, 130, 246, 0.4)',
                black: '#1e1e1e',
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
            fontFamily: '"Fira Code", Consolas, "Courier New", monospace',
            fontSize: 13,
            cursorBlink: true,
            cursorStyle: 'block',
            allowProposedApi: true,
            scrollback: 10000
        });

        this.term.open(container);

        if (window.FitAddon) {
            this.fitAddon = new window.FitAddon.FitAddon();
            this.term.loadAddon(this.fitAddon);
            setTimeout(() => { try { this.fitAddon.fit(); } catch (ignore) { } }, 100);
        }
        window.addEventListener('resize', () => {
            if (this.fitAddon) { try { this.fitAddon.fit(); } catch (ignore) { } }
        });
        
        this.term.onData(data => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(data);
            }
        });

        this.term.onResize(({ cols, rows }) => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(`resize ${cols} ${rows}`);
            }
        });

        this.connect();
    },

    generateSessionId: function() {
        return 'term-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
    },

    // Auth token used for API + WS upgrades.
    token: function() {
        return (
            localStorage.getItem('gb-access-token') ||
            sessionStorage.getItem('gb-access-token') ||
            localStorage.getItem('gb_token') ||
            sessionStorage.getItem('gb_token') ||
            ''
        );
    },

    // Collect the signed-in user's VM containers across their Vibe
    // projects (never the botserver host). Prefers a running dev VM.
    async resolveUserVm() {
        const headers = { 'Content-Type': 'application/json' };
        const tok = this.token();
        if (tok) headers['Authorization'] = 'Bearer ' + tok;
        const api = async (url) => {
            const r = await fetch(url, { headers });
            return r.ok ? r.json() : null;
        };
        try {
            const projects = await api('/api/vibe/projects');
            const list = (projects && projects.projects) || [];
            const vms = [];
            for (const p of list.slice(0, 20)) {
                if (!p || !p.id) continue;
                const data = await api('/api/vibe/projects/' + encodeURIComponent(p.id) + '/vms');
                const rows = (data && data.vms) || [];
                rows.forEach((vm) => { if (vm && vm.container_name) vms.push(vm); });
            }
            if (!vms.length) return null;
            const running = vms.filter((v) => String(v.status || '').indexOf('run') !== -1);
            const preferred =
                running.find((v) => String(v.env || '').indexOf('development') !== -1) ||
                running[0] || vms[0];
            return preferred.container_name;
        } catch (e) {
            console.error('resolveUserVm failed:', e);
            return null;
        }
    },

    // Open a session inside the user's VM. If no VM exists yet the terminal
    // refuses to open a botserver host shell (safety: never expose the
    // server filesystem) and prints instructions instead.
    connect: function() {
        this.sessionId = this.generateSessionId();
        this.term.write('\x1b[36mResolving your VM…\x1b[0m\r\n');

        this.resolveUserVm().then((container) => {
            if (!container) {
                this.term.write('\r\n\x1b[31mNo VM found for your account.\x1b[0m\r\n');
                this.term.write('\x1b[33mOpen Vibe, create a project and publish it to provision a VM, then reopen this terminal.\x1b[0m\r\n');
                return;
            }
            this.createSession(container);
        }).catch(() => {
            this.term.write('\r\n\x1b[31mCould not resolve your VM. Please try again.\x1b[0m\r\n');
        });
    },

    createSession: function(container) {
        const tok = this.token();
        const headers = { 'Content-Type': 'application/json' };
        if (tok) headers['Authorization'] = 'Bearer ' + tok;
        this.term.write('\x1b[36mConnecting to VM ' + container + '…\x1b[0m\r\n');
        fetch('/api/terminal/create', {
            method: 'POST',
            headers,
            body: JSON.stringify({ container: container }),
        }).then((r) => r.json()).then((data) => {
            if (!data || !data.id) {
                this.term.write('\r\n\x1b[31mTerminal session failed: ' + ((data && data.error) || 'unknown error') + '\x1b[0m\r\n');
                return;
            }
            this.sessionId = data.id;
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = protocol + '//' + window.location.host + '/api/terminal/ws?id=' +
                encodeURIComponent(data.id) + (tok ? '&token=' + encodeURIComponent(tok) : '');
            this.ws = new WebSocket(wsUrl);
            this.ws.onopen = () => {
                this.reconnectAttempts = 0;
                this.term.write('\x1b[32m✓ Connected to VM terminal (' + container + ')\x1b[0m\r\n\r\n');
                if (this.term) this.term.focus();
            };
            this.ws.onmessage = (event) => {
                try {
                    const msg = JSON.parse(event.data);
                    if (msg.type === 'connected') {
                        this.term.write('\x1b[33mContainer: ' + (msg.container || container) + '\x1b[0m\r\n');
                    } else if (msg.type === 'system') {
                        this.term.write('\x1b[90m' + msg.message + '\x1b[0m');
                    } else if (msg.type === 'error') {
                        this.term.write('\x1b[31mError: ' + msg.message + '\x1b[0m\r\n');
                    } else if (msg.data != null) {
                        this.term.write(msg.data);
                    }
                } catch (e) {
                    this.term.write(event.data);
                }
            };
            this.ws.onerror = () => {
                console.error('Terminal WebSocket error');
                this.term.write('\x1b[31mConnection error. Attempting to reconnect...\x1b[0m\r\n');
            };
            this.ws.onclose = () => {
                this.term.write('\x1b[33m\x1b[1mDisconnected from terminal.\x1b[0m\r\n');
                if (this.reconnectAttempts < this.maxReconnectAttempts) {
                    this.reconnectAttempts++;
                    setTimeout(() => this.createSession(container), 2000 * this.reconnectAttempts);
                }
            };
        }).catch((err) => {
            this.term.write('\r\n\x1b[31mTerminal unavailable: ' + String(err) + '\x1b[0m\r\n');
        });
    },

    newTerminal: function() {
        if (this.ws) {
            this.ws.close();
        }
        this.connect();
    },

    closeTerminal: function() {
        if (this.ws) {
            this.ws.send('\\exit');
            this.ws.close();
        }
    },

    clearTerminal: function() {
        if (this.term) {
            this.term.clear();
        }
    },

    reconnect: function() {
        this.reconnectAttempts = 0;
        if (this.ws) {
            this.ws.close();
        }
        this.connect();
    }
};

(function(){ var __cb = () => botCoderTerminal.init(); if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

window.botCoderTerminal = botCoderTerminal;
