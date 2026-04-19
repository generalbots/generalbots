const botCoderTerminal = {
    term: null,
    ws: null,
    sessionId: null,
    reconnectAttempts: 0,
    maxReconnectAttempts: 5,

    init: function() {
        if (!window.Terminal) {
            console.error('xterm.js not loaded. Cannot init terminal.');
            document.getElementById('xtermContainer').innerHTML = '<div class="botcoder-error">Terminal library not found. Run "npm install xterm" to install it.</div>';
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

        this.term.open(document.getElementById('xtermContainer'));
        
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

    connect: function() {
        this.sessionId = this.generateSessionId();
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/api/terminal/ws?session_id=${this.sessionId}`;

        this.term.write('\x1b[36mConnecting to isolated terminal...\x1b[0m\r\n');

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            this.reconnectAttempts = 0;
            this.term.write('\x1b[32m✓ Connected to isolated container terminal\x1b[0m\r\n\r\n');
        };

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === 'connected') {
                    this.term.write(`\x1b[33mContainer: ${data.container}\x1b[0m\r\n`);
                    this.term.write(`\x1b[90mSession: ${data.session_id}\x1b[0m\r\n\r\n`);
                } else if (data.type === 'system') {
                    this.term.write(`\x1b[90m${data.message}\x1b[0m`);
                } else if (data.type === 'error') {
                    this.term.write(`\x1b[31mError: ${data.message}\x1b[0m\r\n`);
                }
            } catch (e) {
                this.term.write(event.data);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.term.write('\x1b[31mConnection error. Attempting to reconnect...\x1b[0m\r\n');
        };

        this.ws.onclose = () => {
            this.term.write('\x1b[33m\x1b[1mDisconnected from terminal.\x1b[0m\r\n');
            this.term.write('\x1b[90mType "reconnect" to start a new session\x1b[0m\r\n');
            
            if (this.reconnectAttempts < this.maxReconnectAttempts) {
                this.reconnectAttempts++;
                setTimeout(() => this.connect(), 2000 * this.reconnectAttempts);
            }
        };
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

document.addEventListener('DOMContentLoaded', () => botCoderTerminal.init());
if (document.readyState === 'complete' || document.readyState === 'interactive') {
    botCoderTerminal.init();
}

window.botCoderTerminal = botCoderTerminal;
