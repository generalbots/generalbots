/**
 * Vibe Shell (Terminal) dialog — real PTY over WebSocket.
 * Creates a terminal via /api/terminal/create and attaches xterm.js
 * to /api/terminal/ws (vendored local bundle, no CDN).
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var term = null;
    var fitAddon = null;
    var ws = null;
    var sessionId = null;

    function buildTerm(container) {
        if (!window.Terminal) {
            container.innerHTML = '<div class="vibe-empty">xterm.js not loaded.</div>';
            return;
        }
        term = new window.Terminal({
            theme: {
                background: "#0b0e17",
                foreground: "#e2e8f0",
                cursor: "#4a6cf7",
                selectionBackground: "rgba(74, 108, 247, 0.4)",
                black: "#1e1e1e", red: "#ef4444", green: "#22c55e",
                yellow: "#eab308", blue: "#3b82f6", magenta: "#a855f7",
                cyan: "#06b6d4", white: "#f8fafc",
                brightBlack: "#64748b", brightRed: "#f87171",
                brightGreen: "#4ade80", brightYellow: "#facc15",
                brightBlue: "#60a5fa", brightMagenta: "#c084fc",
                brightCyan: "#22d3ee", brightWhite: "#ffffff",
            },
            fontFamily: '"Fira Code", Consolas, monospace',
            fontSize: 12,
            cursorBlink: true,
            scrollback: 10000,
        });
        if (window.FitAddon) {
            fitAddon = new window.FitAddon.FitAddon();
            term.loadAddon(fitAddon);
        }
        term.open(container);
        setTimeout(fit, 120);

        term.onData(function (data) {
            if (ws && ws.readyState === WebSocket.OPEN) ws.send(data);
        });
        term.onResize(function (dim) {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send("resize " + dim.cols + " " + dim.rows);
            }
        });
        connect();
    }

    function fit() {
        if (fitAddon) {
            try { fitAddon.fit(); } catch (ignore) { }
        }
    }

    function connect() {
        if (!term) return;
        sessionId = "vibe-" + Date.now() + "-" + Math.random().toString(36).substr(2, 8);

        D.api("/api/terminal/create", {
            method: "POST",
            body: { cwd: "/tmp/vibe-workspaces", shell: "/bin/bash" },
        }).then(function (data) {
            if (data && data.id) sessionId = data.id;
            var proto = location.protocol === "https:" ? "wss:" : "ws:";
            var token =
                localStorage.getItem("gb-access-token") ||
                sessionStorage.getItem("gb-access-token") ||
                "";
            var url = proto + "//" + location.host + "/api/terminal/ws?id=" +
                encodeURIComponent(sessionId) +
                (token ? "&token=" + encodeURIComponent(token) : "");
            ws = new WebSocket(url);
            ws.onopen = function () {
                // The PTY shell prints its own prompt — no client-side prompt
                // is needed (a fake prompt would double up with the real one).
                term.write("\x1b[32m✓ connected — vibe workspace shell\x1b[0m\r\n");
            };
            ws.onmessage = function (ev) {
                try {
                    var msg = JSON.parse(ev.data);
                    if (msg.type === "connected") {
                        term.write("\r\n\x1b[90m" + (msg.message || "") + "\x1b[0m\r\n");
                    } else if (msg.type === "system") {
                        term.write("\x1b[90m" + msg.message + "\x1b[0m");
                    } else if (msg.type === "error") {
                        term.write("\x1b[31m" + msg.message + "\x1b[0m\r\n");
                    } else if (msg.data != null) {
                        term.write(msg.data);
                    }
                } catch (e) {
                    term.write(ev.data);
                }
            };
            ws.onclose = function () {
                term.write("\r\n\x1b[33mdisconnected\x1b[0m\r\n");
            };
            ws.onerror = function () {
                term.write("\r\n\x1b[31mconnection error — retrying…\x1b[0m\r\n");
                setTimeout(connect, 3000);
            };
        }).catch(function () {
            term.write("\r\n\x1b[31mterminal backend unavailable\x1b[0m\r\n");
        });
    }

    D.register("terminal", {
        build: function (body) {
            var wrap = D.el("div", "vibe-term-wrap");
            wrap.id = "vibeTermHost";
            body.appendChild(wrap);
            buildTerm(wrap);
            var resizeTimer = null;
            window.addEventListener("resize", function () {
                clearTimeout(resizeTimer);
                resizeTimer = setTimeout(fit, 150);
            });
        },
        teardown: function () {
            if (ws) {
                try { ws.close(); } catch (ignore) { }
                ws = null;
            }
            if (term) {
                try { term.dispose(); } catch (ignore) { }
                term = null;
            }
            fitAddon = null;
        },
    });
})();