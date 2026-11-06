(function () {
'use strict';
var VDI = window.VDI;
var toast = VDI.toast;
var switchView = VDI.switchView;
var drawStatusScreen = VDI.drawStatusScreen;
var activeSessions = new Map();
var MAX_SESSIONS = 3;

    function updateSessionBadge() {
        var badge = document.getElementById("session-count");
        if (badge) {
            badge.textContent = activeSessions.size + "/" + MAX_SESSIONS + " sessions";
            badge.className = "vdi-session-badge" + (activeSessions.size >= MAX_SESSIONS ? " full" : "");
        }
    }

    async function startSession(host, port, protocol, password) {
        protocol = protocol || "vnc";
        // The backend only accepts registered (UUID) sessions — quick connect
        // must register via POST /connect first, then proxy through the returned id.
        var sessionId;
        var payload = { name: "Desktop", host: host, port: port, protocol: protocol, auth_type: "none" };
        if (password) {
            payload.auth_type = "password";
            payload.password = password;
        }
        try {
            var reg = await fetch("/api/desktop/connect", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            if (!reg.ok) throw new Error("registration failed");
            var regData = await reg.json();
            sessionId = (regData.data && regData.data.id) || (regData.data && regData.data.connection_id) || regData.id;
            if (!sessionId) throw new Error("no session id returned");
        } catch (e) {
            toast("Failed to register connection: " + e.message, "error");
            return;
        }
        startSessionWithId(host, port, sessionId, protocol);
    }

    async function startSessionWithId(host, port, sessionId, protocol) {
        protocol = protocol || "vnc";
        if (!sessionId) {
            toast("No session id available", "error");
            return;
        }
        if (activeSessions.size >= MAX_SESSIONS) {
            toast("Maximum " + MAX_SESSIONS + " concurrent sessions", "error");
            return;
        }

        var wsProto = location.protocol === "https:" ? "wss:" : "ws:";
        var wsUrl = wsProto + "//" + location.host + "/api/desktop/ws/proxy/" + sessionId;

        var statusEl = document.getElementById("connection-status");
        var infoEl = document.getElementById("connection-info");
        var loadingEl = document.getElementById("vnc-loading");
        var containerEl = document.getElementById("vnc-container");

        if (statusEl) {
            statusEl.textContent = "Connecting...";
            statusEl.className = "status-badge status-connecting";
        }
        if (infoEl) infoEl.textContent = host + ":" + port + " (" + protocol.toUpperCase() + ")";
        if (loadingEl) loadingEl.style.display = "";

        switchView("canvas");

        var session = {
            adapter: null,
            ws: null,
            host: host,
            port: port,
            protocol: protocol,
            connected: false,
        };
        activeSessions.set(sessionId, session);
        updateSessionBadge();

        if (protocol === "rdp" && typeof RDPAdapter !== "undefined") {
            connectWithRDP(sessionId, session, wsUrl, host, port);
        } else if (typeof VNCAdapter !== "undefined") {
            connectWithNoVNC(sessionId, session, wsUrl, host, port);
        } else {
            connectRawProxy(sessionId, session, wsUrl, host, port);
        }

        document.getElementById("btn-disconnect").onclick = function () {
            disconnectSession(sessionId);
            switchView("dashboard");
        };

        document.getElementById("btn-fullscreen").onclick = function () {
            var vc = document.getElementById("vnc-container");
            if (!document.fullscreenElement && vc) {
                vc.requestFullscreen();
            } else if (document.fullscreenElement) {
                document.exitFullscreen();
            }
        };

        document.getElementById("btn-clipboard").onclick = function () {
            if (session.adapter && session.connected) {
                navigator.clipboard.readText().then(function (text) {
                    session.adapter.clipboardPaste(text);
                    toast("Clipboard sent", "info");
                }).catch(function () {
                    toast("Clipboard access denied", "error");
                });
            } else {
                toast("Clipboard requires an active connection", "info");
            }
        };

        document.getElementById("btn-ctrl-alt-del").onclick = function () {
            if (session.adapter && session.connected) {
                session.adapter.sendCtrlAltDel();
                toast("Ctrl+Alt+Del sent", "info");
            } else {
                toast("Ctrl+Alt+Del requires an active connection", "info");
            }
        };
    }

    function connectWithNoVNC(sessionId, session, wsUrl, host, port) {
        var container = document.getElementById("vnc-container");
        var loadingEl = document.getElementById("vnc-loading");

        var adapter = new VNCAdapter(container, wsUrl, host, port);

        adapter.addEventListener("connect", function () {
            if (loadingEl) loadingEl.style.display = "none";
            var statusEl = document.getElementById("connection-status");
            if (statusEl) {
                statusEl.textContent = "Connected";
                statusEl.className = "status-badge status-connected";
            }
            session.connected = true;
            session.adapter = adapter;
            toast("Connected to " + host + ":" + port, "success");
        });

        adapter.addEventListener("disconnect", function () {
            var statusEl = document.getElementById("connection-status");
            if (statusEl) {
                statusEl.textContent = "Disconnected";
                statusEl.className = "status-badge status-disconnected";
            }
            disconnectSession(sessionId);
        });

        adapter.addEventListener("error", function (e) {
            var statusEl = document.getElementById("connection-status");
            if (statusEl) {
                statusEl.textContent = "Error";
                statusEl.className = "status-badge status-error";
            }
            toast("VNC error: " + (e.detail || "unknown"), "error");
            if (loadingEl) loadingEl.style.display = "none";
            connectRawProxy(sessionId, session, wsUrl, host, port);
        });

        session.adapter = adapter;
        adapter.connect();
    }

    function connectWithRDP(sessionId, session, wsUrl, host, port) {
        var loadingEl = document.getElementById("vnc-loading");

        var adapter = new RDPAdapter(document.getElementById("vnc-container"), wsUrl, host, port);

        adapter.addEventListener("connect", function (e) {
            if (loadingEl) loadingEl.style.display = "none";
            var statusEl = document.getElementById("connection-status");
            if (statusEl) {
                statusEl.textContent = "Connected";
                statusEl.className = "status-badge status-connected";
            }
            session.connected = true;
            session.adapter = adapter;
            toast("Connected to " + host + ":" + port + " (RDP)", "success");
        });

        adapter.addEventListener("disconnect", function () {
            var statusEl = document.getElementById("connection-status");
            if (statusEl) {
                statusEl.textContent = "Disconnected";
                statusEl.className = "status-badge status-disconnected";
            }
            disconnectSession(sessionId);
        });

        adapter.addEventListener("error", function (e) {
            var statusEl = document.getElementById("connection-status");
            if (statusEl) {
                statusEl.textContent = "Error";
                statusEl.className = "status-badge status-error";
            }
            toast("RDP error: " + (e.detail || "unknown"), "error");
            if (loadingEl) loadingEl.style.display = "none";
        });

        session.adapter = adapter;
        adapter.connect();
    }

    function connectRawProxy(sessionId, session, wsUrl, host, port) {
        var ws;
        try {
            ws = new WebSocket(wsUrl);
            ws.binaryType = "arraybuffer";
        } catch (e) {
            toast("WebSocket connection failed", "error");
            disconnectSession(sessionId);
            return;
        }

        session.ws = ws;

        var statusEl = document.getElementById("connection-status");
        var loadingEl = document.getElementById("vnc-loading");
        var canvasEl = document.getElementById("vnc-canvas");

        ws.onopen = function () {
            ws.send(JSON.stringify({ host: host, port: port }));
        };

        var frameCount = 0;

        ws.onmessage = function (event) {
            if (typeof event.data === "string") {
                try {
                    var msg = JSON.parse(event.data);
                    if (msg.status === "connected") {
                        if (loadingEl) loadingEl.style.display = "none";
                        if (statusEl) {
                            statusEl.textContent = "Connected (raw proxy)";
                            statusEl.className = "status-badge status-connected";
                        }
                        session.connected = true;
                        if (canvasEl) {
                            drawStatusScreen(canvasEl, host, port, "TCP tunnel established. Waiting for VNC data...");
                        }
                        toast("TCP tunnel established", "success");
                    } else if (msg.error) {
                        if (statusEl) {
                            statusEl.textContent = "Error";
                            statusEl.className = "status-badge status-error";
                        }
                        if (loadingEl) loadingEl.style.display = "none";
                        if (canvasEl) {
                            drawStatusScreen(canvasEl, host, port, "Error: " + msg.error);
                        }
                        toast("Connection error: " + msg.error, "error");
                        disconnectSession(sessionId);
                    }
                } catch (e) { /* ignore */ }
                return;
            }

            if (event.data instanceof ArrayBuffer) {
                frameCount++;
                if (loadingEl) loadingEl.style.display = "none";
                if (statusEl) {
                    statusEl.textContent = "Connected - " + frameCount + " frames";
                    statusEl.className = "status-badge status-connected";
                }
                session.connected = true;
                if (frameCount === 1 && canvasEl) {
                    drawStatusScreen(canvasEl, host, port, "Receiving VNC data (" + event.data.byteLength + " bytes)...");
                }
            }
        };

        ws.onclose = function () {
            if (statusEl) {
                statusEl.textContent = "Disconnected";
                statusEl.className = "status-badge status-disconnected";
            }
            disconnectSession(sessionId);
        };

        ws.onerror = function () {
            if (statusEl) {
                statusEl.textContent = "Error";
                statusEl.className = "status-badge status-error";
            }
            toast("WebSocket error", "error");
            disconnectSession(sessionId);
        };
    }

    function disconnectSession(sessionId) {
        var session = activeSessions.get(sessionId);
        if (session) {
            if (session.adapter) {
                try { session.adapter.disconnect(); } catch (e) { /* ignore */ }
            }
            if (session.ws) {
                try { session.ws.close(); } catch (e) { /* ignore */ }
            }
            activeSessions.delete(sessionId);

            var rfbDiv = document.getElementById("rfb-container");
            if (rfbDiv) rfbDiv.remove();

            var canvasEl = document.getElementById("vnc-canvas");
            if (canvasEl) {
                canvasEl.style.display = "";
                var ctx = canvasEl.getContext("2d");
                if (ctx) ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);
            }
        }
        updateSessionBadge();
    }

    VDI.startSession = startSession;
    VDI.startSessionWithId = startSessionWithId;
})();
