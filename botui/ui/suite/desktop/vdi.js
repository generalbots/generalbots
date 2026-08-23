(function() {
'use strict';
if (window.GBAppLifecycle) GBAppLifecycle.begin("vdi");
var activeSessions = new Map();
    var MAX_SESSIONS = 3;
    var CONNECTIONS = [];

    function esc(s) {
        var d = document.createElement("div");
        d.textContent = s || "";
        return d.innerHTML;
    }

    function toast(msg, type) {
        var container = document.getElementById("toast-container");
        if (!container) return;
        var el = document.createElement("div");
        el.className = "toast toast-" + (type || "info");
        el.textContent = msg;
        container.appendChild(el);
        setTimeout(function () {
            el.classList.add("toast-fade");
            setTimeout(function () { el.remove(); }, 300);
        }, 3000);
    }

    function showModal(title, bodyHtml) {
        var overlay = document.getElementById("modal-overlay");
        var titleEl = document.getElementById("modal-title");
        var bodyEl = document.getElementById("modal-body");
        if (titleEl) titleEl.textContent = title;
        if (bodyEl) bodyEl.innerHTML = bodyHtml;
        if (overlay) overlay.classList.remove("hidden");
    }

    function hideModal() {
        var overlay = document.getElementById("modal-overlay");
        if (overlay) overlay.classList.add("hidden");
    }

    function updateSessionBadge() {
        var badge = document.getElementById("session-count");
        if (badge) {
            badge.textContent = activeSessions.size + "/" + MAX_SESSIONS + " sessions";
            badge.className = "vdi-session-badge" + (activeSessions.size >= MAX_SESSIONS ? " full" : "");
        }
    }

    function updateConnectionCount() {
        var badge = document.getElementById("connections-count");
        if (badge) badge.textContent = CONNECTIONS.length;
    }

    function switchView(view) {
        var dashboard = document.getElementById("vdi-dashboard");
        var canvas = document.getElementById("vdi-canvas");
        if (view === "canvas") {
            if (dashboard) dashboard.classList.add("hidden");
            if (canvas) canvas.classList.remove("hidden");
        } else {
            if (dashboard) dashboard.classList.remove("hidden");
            if (canvas) canvas.classList.add("hidden");
        }
    }

    async function loadConnections() {
        try {
            var resp = await fetch("/api/desktop/connections");
            if (!resp.ok) return;
            var body = await resp.json();
            CONNECTIONS = (body && body.data) || [];
            renderConnections();
        } catch (e) {
            console.error("Failed to load connections:", e);
        }
    }

    function renderConnections() {
        var grid = document.getElementById("connections-grid");
        if (!grid) return;
        updateConnectionCount();
        if (!CONNECTIONS.length) {
            grid.innerHTML =
                '<div class="vdi-empty-state">' +
                '<div class="empty-icon">&#128421;</div>' +
                '<h3>No connections yet</h3>' +
                '<p>Create a new connection to get started</p>' +
                "</div>";
            return;
        }
        grid.innerHTML = CONNECTIONS.map(function (c) {
            var host = c.host || c.target_host || "";
            var port = c.port || c.target_port || 5900;
            var name = c.name || "Desktop";
            return (
                '<div class="vdi-connection-card" data-id="' + esc(c.id) + '">' +
                '<div class="card-header">' +
                '<span class="card-icon">&#128187;</span>' +
                '<div class="card-info">' +
                '<div class="card-name">' + esc(name) + "</div>" +
                '<div class="card-host">' + esc(host) + ":" + port + "</div>" +
                "</div>" +
                "</div>" +
                '<div class="card-footer">' +
                '<span class="card-protocol">' + esc(c.protocol || "vnc").toUpperCase() + "</span>" +
                '<div class="card-actions">' +
                '<button class="btn btn-primary btn-xs" onclick="VDI.connectSaved(\'' + esc(c.id) + "')\">Connect</button>" +
                '<button class="btn btn-ghost btn-xs btn-danger" onclick="VDI.deleteSaved(\'' + esc(c.id) + "')\">&#10005;</button>" +
                "</div>" +
                "</div>" +
                "</div>"
            );
        }).join("");
    }

    function drawStatusScreen(canvas, host, port, statusText) {
        var ctx = canvas.getContext("2d");
        var w = canvas.width = canvas.parentElement.clientWidth || 800;
        var h = canvas.height = canvas.parentElement.clientHeight || 600;
        ctx.fillStyle = "#1a1b26";
        ctx.fillRect(0, 0, w, h);
        ctx.fillStyle = "#7aa2f7";
        ctx.font = "bold 20px monospace";
        ctx.textAlign = "center";
        ctx.fillText("Desktop VDI", w / 2, h / 2 - 60);
        ctx.fillStyle = "#565f89";
        ctx.font = "14px monospace";
        ctx.fillText("Target: " + host + ":" + port, w / 2, h / 2 - 20);
        ctx.fillStyle = "#9ece6a";
        ctx.font = "16px monospace";
        ctx.fillText(statusText || "Connecting...", w / 2, h / 2 + 20);
        ctx.fillStyle = "#565f89";
        ctx.font = "12px monospace";
        ctx.fillText("VNC rendering requires noVNC library", w / 2, h / 2 + 60);
        ctx.fillText("Place noVNC files in /suite/desktop/vendor/novnc/", w / 2, h / 2 + 80);
    }

    async function startSession(host, port, protocol) {
        protocol = protocol || "vnc";
        // The backend only accepts registered (UUID) sessions — quick connect
        // must register via POST /connect first, then proxy through the returned id.
        var sessionId;
        try {
            var reg = await fetch("/api/desktop/connect", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ name: "Desktop", host: host, port: port, protocol: protocol, auth_type: "password" }),
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

    function showNewConnectionForm() {
        var formHtml =
            '<div class="vdi-form">' +
            '<div class="form-group">' +
            '<label>Connection Name</label>' +
            '<input type="text" id="new-conn-name" class="input" placeholder="My Server">' +
            "</div>" +
            '<div class="form-group">' +
            '<label>Host</label>' +
            '<input type="text" id="new-conn-host" class="input" placeholder="10.0.0.1">' +
            "</div>" +
            '<div class="form-group">' +
            '<label>Port</label>' +
            '<input type="number" id="new-conn-port" class="input" value="5900">' +
            "</div>" +
            '<div class="form-group">' +
            '<label>Protocol</label>' +
            '<select id="new-conn-protocol" class="input" onchange="VDI.onProtocolChange()">' +
            '<option value="vnc">VNC</option>' +
            '<option value="rdp">RDP</option>' +
            "</select>" +
            "</div>" +
            '<div class="form-group" id="new-conn-rdp-fields" style="display:none;">' +
            '<label>RDP Password</label>' +
            '<input type="password" id="new-conn-password" class="input" placeholder="Target password (vaulted)" autocomplete="off">' +
            "</div>" +
            '<div class="form-group" id="new-conn-rdp-domain" style="display:none;">' +
            '<label>RDP Domain (optional)</label>' +
            '<input type="text" id="new-conn-domain" class="input" placeholder="CORP\\user">' +
            "</div>" +
            '<div class="form-actions">' +
            '<button class="btn btn-ghost" onclick="VDI.closeModal()">Cancel</button>' +
            '<button class="btn btn-primary" onclick="VDI.saveNewConnection()">Save & Connect</button>' +
            "</div>" +
            "</div>";
        showModal("New Connection", formHtml);
        window.VDI.onProtocolChange = function () {
            var proto = (document.getElementById("new-conn-protocol") || {}).value || "vnc";
            var portEl = document.getElementById("new-conn-port");
            var pwdRow = document.getElementById("new-conn-rdp-fields");
            var domRow = document.getElementById("new-conn-rdp-domain");
            var isRdp = proto === "rdp";
            if (portEl && isRdp && parseInt(portEl.value, 10) === 5900) portEl.value = "3389";
            if (portEl && !isRdp && parseInt(portEl.value, 10) === 3389) portEl.value = "5900";
            if (pwdRow) pwdRow.style.display = isRdp ? "" : "none";
            if (domRow) domRow.style.display = isRdp ? "" : "none";
        };
    }

    window.saveNewConnection = async function saveNewConnection() {
    window.showNewConnectionForm = showNewConnectionForm;
        var name = (document.getElementById("new-conn-name") || {}).value || "";
        var host = (document.getElementById("new-conn-host") || {}).value || "";
        var port = parseInt((document.getElementById("new-conn-port") || {}).value || "5900");
        var protocol = (document.getElementById("new-conn-protocol") || {}).value || "vnc";
        var password = (document.getElementById("new-conn-password") || {}).value || "";
        var domain = (document.getElementById("new-conn-domain") || {}).value || "";
        name = name.trim();
        host = host.trim();
        if (!name || !host) {
            toast("Name and host are required", "error");
            return;
        }
        try {
            var payload = { name: name, host: host, port: port, protocol: protocol, auth_type: "password" };
            if (protocol === "rdp") {
                if (password) payload.password = password;
                if (domain) payload.domain = domain;
            }
            var resp = await fetch("/api/desktop/connect", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload),
            });
            if (!resp.ok) throw new Error("Save failed");
            var saved = await resp.json();
            hideModal();
            toast("Connection saved", "success");
            await loadConnections();
            startSessionWithId(host, port, saved.data && saved.data.id, protocol);
        } catch (e) {
            toast("Failed to save: " + e.message, "error");
        }
    }

})();
