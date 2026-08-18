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
    // The dialog's teardown nulls `term` while a pending create/connect may
    // still resolve afterwards; WS callbacks must not crash on a null term
    // (#912). tornDown also stops the retry loop from resurrecting a closed
    // dialog.
    var tornDown = false;

    function writeTerm(text) {
        if (term) {
            try { term.write(text); } catch (ignore) { }
        }
    }

    function buildTerm(container) {
        tornDown = false;
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

    // Resolve the selected project's VM container so the terminal execs into
    // the project environment (the calculator VM) instead of the botserver
    // host filesystem. Falls back to a host shell when no project/VM exists.
    function resolveContainer() {
        var pid =
            typeof window.currentProjectId !== "undefined" && window.currentProjectId
                ? window.currentProjectId
                : null;
        if (!pid) return Promise.resolve(null);
        return D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/vms")
            .then(function (data) {
                var vms = (data && (data.vms || (data.success && data.data && data.data.vms))) || [];
                if (!Array.isArray(vms) || !vms.length) return null;
                // Prefer a running dev VM; otherwise the first one.
                var preferred = vms.find(function (v) {
                    return String(v.env || "").indexOf("development") !== -1 &&
                        String(v.status || "").indexOf("run") !== -1;
                }) || vms.find(function (v) {
                    return String(v.status || "").indexOf("run") !== -1;
                }) || vms[0];
                return (preferred && preferred.container_name) ? preferred.container_name : null;
            })
            .catch(function () { return null; });
    }

    function connect() {
        if (!term || tornDown) return;
        sessionId = "vibe-" + Date.now() + "-" + Math.random().toString(36).substr(2, 8);

        resolveContainer().then(function (container) {
            if (tornDown) return;
            var body = { cwd: "/tmp/vibe-workspaces", shell: "/bin/bash" };
            if (container) body.container = container;
            return D.api("/api/terminal/create", {
                method: "POST",
                body: body,
            });
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
                // The dialog may have been closed while the WS connected;
                // bail so we never write to a disposed terminal.
                if (!term) return;
                // The PTY shell prints its own prompt — no client-side prompt
                // is needed (a fake prompt would double up with the real one).
                writeTerm("\x1b[32m✓ connected — vibe workspace shell\x1b[0m\r\n");
                // Grab focus so the user can type immediately after the
                // dialog opens — without it, keystrokes go nowhere until
                // the terminal is clicked once.
                if (term) term.focus();
            };
            ws.onmessage = function (ev) {
                if (!term) return;
                try {
                    var msg = JSON.parse(ev.data);
                    if (msg.type === "connected") {
                        writeTerm("\r\n\x1b[90m" + (msg.message || "") + "\x1b[0m\r\n");
                    } else if (msg.type === "system") {
                        writeTerm("\x1b[90m" + msg.message + "\x1b[0m");
                    } else if (msg.type === "error") {
                        writeTerm("\x1b[31m" + msg.message + "\x1b[0m\r\n");
                    } else if (msg.data != null) {
                        writeTerm(msg.data);
                    }
                } catch (e) {
                    writeTerm(ev.data);
                }
            };
            ws.onclose = function () {
                writeTerm("\r\n\x1b[33mdisconnected\x1b[0m\r\n");
            };
            ws.onerror = function () {
                writeTerm("\r\n\x1b[31mconnection error — retrying…\x1b[0m\r\n");
                setTimeout(function () {
                    if (!tornDown) connect();
                }, 3000);
            };
        }).catch(function () {
            writeTerm("\r\n\x1b[31mterminal backend unavailable\x1b[0m\r\n");
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
            tornDown = true;
            if (ws) {
                try { ws.close(); } catch (ignore) { }
                ws = null;
            }
            // Null the global BEFORE dispose so any in-flight ws.onmessage /
            // onopen callback that fires during teardown sees term === null and
            // bails instead of calling write() on a half-disposed terminal.
            var oldTerm = term;
            term = null;
            fitAddon = null;
            if (oldTerm) {
                try { oldTerm.dispose(); } catch (ignore) { }
            }
        },
    });
})();