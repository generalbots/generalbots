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

    // Dispose the previous terminal session, socket and xterm instance. A
    // re-opened dialog MUST NOT keep the old PTY alive: every live session
    // prints its own `root@name>` prompt into the same terminal, which is
    // exactly the "prompt repeated 3 times" the user saw after opening the
    // Terminal three times. Only one session may be attached at a time.
    function disposeTerminal() {
        tornDown = true;
        if (ws) {
            try { ws.close(); } catch (ignore) { }
            ws = null;
        }
        var oldTerm = term;
        term = null;
        fitAddon = null;
        if (oldTerm) {
            try { oldTerm.dispose(); } catch (ignore) { }
        }
    }

    function buildTerm(container) {
        // Kill any previous session BEFORE building a new one so prompts
        // from stale PTYs never stack on top of the fresh shell.
        disposeTerminal();
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

    // Resolve the selected project's terminal target: exec into the project
    // VM container. SECURITY: never fall back to a shell on the botserver
    // host — the previous workspace-cwd fallback opened the server's own
    // filesystem (the backend now refuses container-less sessions too).
    function resolveTarget() {
        // Same source as the rest of the shell (S.projectId): the in-memory
        // selection first, then the persisted one so a terminal still works
        // right after a page reload without re-selecting the project.
        var pid =
            (typeof window.currentProjectId !== "undefined" && window.currentProjectId) ||
            (function () {
                try { return sessionStorage.getItem("gb-vibe-project-id") || ""; } catch (e) { return ""; }
            })() ||
            null;
        if (!pid) return Promise.resolve({ container: null, error: "No project selected — open a project first." });
        return D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/vms")
            .then(function (data) {
                if (data && data.success === false) throw new Error(data.error || "VM lookup failed");
                var vms = (data && (data.vms || (data.success && data.data && data.data.vms))) || [];
                if (Array.isArray(vms) && vms.length) {
                    // #1288 — only a RUNNING VM may be attached: incus exec
                    // into a stopped/creating container blocks forever. A
                    // non-running VM surfaces the "start it (Play)" hint
                    // instead of hanging the dialog on a silent shell.
                    var preferred = vms.find(function (v) {
                        return String(v.env || "").indexOf("development") !== -1 &&
                            String(v.status || "").indexOf("run") !== -1;
                    }) || vms.find(function (v) {
                        return String(v.status || "").indexOf("run") !== -1;
                    }) || null;
                    if (preferred && preferred.container_name) {
                        return { container: preferred.container_name };
                    }
                }
                // No running VM: refuse rather than expose a host shell.
                return { container: null, error: "The project VM is not running — start it (▶ Play) to open a terminal inside it." };
            })
            .catch(function (error) {
                throw new Error("Could not resolve the project: " + (error && error.message ? error.message : error));
            });
    }

    function connect() {
        if (!term || tornDown) return;
        sessionId = "vibe-" + Date.now() + "-" + Math.random().toString(36).substr(2, 8);

        resolveTarget().then(function (target) {
            if (tornDown) return;
            // Exec into the project VM when available; otherwise root the
            // shell at the project workspace (never the whole host).
            var createBody = {};
            if (target && target.container) {
                createBody.container = target.container;
            } else {
                writeTerm("\r\n\x1b[31m" + (target && target.error ? target.error : "No project VM available — start it first.") + "\x1b[0m\r\n");
                return null;
            }
            return D.api("/api/terminal/create", {
                method: "POST",
                body: createBody,
            });
        }).then(function (data) {
            if (!data) return; // host-shell refusal already reported
            if (!data.id) {
                throw new Error((data && data.error) || "The terminal backend did not create a session");
            }
            sessionId = data.id;
            var proto = location.protocol === "https:" ? "wss:" : "ws:";
            var token =
                (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
                localStorage.getItem("token") ||
                localStorage.getItem("id_token") ||
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
                // Leading \r\n separates the banner from any prior output so
                // a new shell's prompt always starts on a fresh line.
                writeTerm("\r\n\x1b[32mconnected — project WSL/Incus workspace\x1b[0m\r\n");
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
        }).catch(function (error) {
            writeTerm("\r\n\x1b[31mterminal unavailable: " + (error && error.message ? error.message : error) + "\x1b[0m\r\n");
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
            // Null the globals BEFORE dispose so any in-flight ws.onmessage /
            // onopen callback that fires during teardown sees term === null
            // and bails instead of calling write() on a half-disposed
            // terminal.
            disposeTerminal();
        },
    });
})();
