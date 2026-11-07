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

    // #1841 — turn the backend's capability categories into instructions the
    // user can act on. The raw payload stays in the console for support.
    errorMessage: function(data, status) {
        var code = (data && (data.error || data.code)) || '';
        var detail = data && (data.message || data.error);
        if (code === 'agent_vm_unavailable') {
            return 'Terminal requires the agent VM, which is not available in this build.';
        }
        if (code === 'no_workspace_vm') {
            return 'No workspace VM is running. Send a message in chat to start your workspace, then reopen the terminal.';
        }
        if (code === 'vm_not_found' || code === 'vm_stopped') {
            return 'Your workspace VM is not running. Send a message in chat to start it, then reopen the terminal.';
        }
        if (code === 'incus_unavailable') {
            return 'Terminal needs Incus, but it is not installed or not available on this machine.';
        }
        if (code === 'terminal_forbidden') {
            return 'The terminal cannot access the workspace VM. Check the Incus permissions.';
        }
        if (code === 'terminal_unavailable') {
            return detail || 'Terminal is not available in this build.';
        }
        if (code === 'not_found') {
            return 'Terminal is not available in this build (the terminal API is not compiled).';
        }
        if (status === 404) {
            return 'Terminal is not available in this build.';
        }
        return detail || ('HTTP ' + status);
    },

    // The product endpoint reports the combined terminal + agent-VM
    // capability. Skip the request entirely when the build cannot host a
    // workspace shell.
    capabilityMessage: function() {
        var cfg = window.productConfig;
        if (cfg && cfg.terminal_available === false) {
            return 'Terminal requires the agent VM, which is not available in this desktop build.';
        }
        return '';
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

    // The chat workspace container is the primary target: it holds the
    // conversation folders the agent just wrote to. Only fall back to a Vibe
    // project VM when the user has no agent workspace yet.
    async resolveAgentVm() {
        const headers = {};
        const tok = this.token();
        if (tok) headers['Authorization'] = 'Bearer ' + tok;
        try {
            const r = await fetch('/api/agent/vm/status', { headers });
            if (!r.ok) return null;
            const data = await r.json();
            return data && data.has_vm && data.vm_name ? data.vm_name : null;
        } catch (e) {
            return null;
        }
    },

    // Open a session inside the user's VM. If no VM exists yet the terminal
    // refuses to open a botserver host shell (safety: never expose the
    // server filesystem) and prints instructions instead.
    connect: function() {
        var unavailable = this.capabilityMessage();
        if (unavailable) {
            this.term.write('\r\n\x1b[31m' + unavailable + '\x1b[0m\r\n');
            this.term.write('\x1b[33mChat and file output keep working; the terminal needs the desktop edition\'s agent VM.\x1b[0m\r\n');
            return;
        }
        this.sessionId = this.generateSessionId();
        this.term.write('\x1b[36mResolving your workspace VM…\x1b[0m\r\n');

        this.resolveAgentVm().then((agentVm) => agentVm || this.resolveUserVm()).then((container) => {
            if (!container) {
                this.term.write('\r\n\x1b[31mNo workspace VM found for your account.\x1b[0m\r\n');
                this.term.write('\x1b[33mSend a message in chat to start your workspace, then reopen this terminal.\x1b[0m\r\n');
                return;
            }
            this.createSession(container);
        }).catch(() => {
            this.term.write('\r\n\x1b[31mCould not resolve your VM. Please try again.\x1b[0m\r\n');
        });
    },

    createSession: function(container) {
        var self = this;
        var tok = this.token();
        var headers = { 'Content-Type': 'application/json' };
        if (tok) headers['Authorization'] = 'Bearer ' + tok;
        this.term.write('\x1b[36mConnecting to VM ' + container + '…\x1b[0m\r\n');
        fetch('/api/terminal/create', {
            method: 'POST',
            headers,
            body: JSON.stringify({ container: container }),
        }).then((r) => {
            return r.text().then((text) => {
                var data = {};
                try { data = text ? JSON.parse(text) : {}; } catch (e) { data = { error: text }; }
                return { ok: r.ok, status: r.status, data: data };
            });
        }).then((result) => {
            var data = result.data;
            if (!result.ok || !data || !data.id) {
                console.error('Terminal create failed:', result.status, data);
                const errCode = data && (data.error || data.detail);
                let errMsg;
                if (errCode === 'not_found' || errCode === 'no_workspace') {
                    errMsg = 'No workspace VM found. Send a message in Chat first to start your workspace, then retry.';
                } else if (errCode === 'forbidden' || (typeof errCode === 'string' && errCode.startsWith('Host shell denied'))) {
                    errMsg = 'Terminal access requires an active workspace. Open Chat and start a task first.';
                } else {
                    errMsg = self.errorMessage(data, result.status);
                }
                this.term.write('\r\n\x1b[31mTerminal session failed — ' + errMsg + '\x1b[0m\r\n');
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
            console.error('Terminal create request failed:', err);
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
