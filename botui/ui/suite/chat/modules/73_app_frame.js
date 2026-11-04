/**
 * Inline MCP Apps frame (#1676).
 *
 * An app is untrusted HTML. It renders in an opaque iframe (Mode A: scripts
 * allowed, no same-origin), speaks JSON-RPC over postMessage and is unloaded
 * when scrolled away. Only this module may create the frame.
 */
(function () {
    "use strict";

    var MAX_LIVE = 2;
    var MAX_MESSAGE_BYTES = 256 * 1024;
    var MAX_MESSAGES_PER_SECOND = 20;
    var live = [];
    var contextStore = Object.create(null);

    function safeTitle(value) {
        return String(value == null ? "App" : value).slice(0, 120);
    }

    function frameOrigin(url) {
        try {
            return new URL(url, window.location.href).origin;
        } catch (e) {
            return "null";
        }
    }

    function createFrameHost(payload) {
        var card = document.createElement("section");
        card.className = "gb-app-card";

        var head = document.createElement("header");
        head.className = "gb-app-head";
        var title = document.createElement("span");
        title.className = "gb-app-title";
        title.textContent = safeTitle(payload.title);
        var status = document.createElement("span");
        status.className = "gb-app-status";
        status.textContent = "Loading…";
        status.setAttribute("role", "status");

        var reload = document.createElement("button");
        reload.type = "button";
        reload.className = "gb-app-btn";
        reload.textContent = "Reload";
        var expand = document.createElement("button");
        expand.type = "button";
        expand.className = "gb-app-btn";
        expand.textContent = "Expand";
        var stop = document.createElement("button");
        stop.type = "button";
        stop.className = "gb-app-btn";
        stop.textContent = "Stop";

        head.appendChild(title);
        head.appendChild(status);
        head.appendChild(reload);
        head.appendChild(expand);
        head.appendChild(stop);

        var body = document.createElement("div");
        body.className = "gb-app-body";

        var poster = document.createElement("button");
        poster.type = "button";
        poster.className = "gb-app-poster";
        poster.textContent = "App paused to save resources. Click to resume.";
        poster.hidden = true;

        card.appendChild(head);
        card.appendChild(body);
        card.appendChild(poster);
        return { card: card, body: body, status: status, poster: poster, payload: payload };
    }

    function mountFrame(host) {
        if (host.frame) return;
        host.poster.hidden = true;
        host.mountedAt = Date.now();
        var payload = host.payload;
        var source = payload.url;
        if (!source && payload.html_base64) {
            try {
                var binary = window.atob(payload.html_base64);
                var bytes = new Uint8Array(binary.length);
                for (var i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
                source = URL.createObjectURL(new Blob([bytes], { type: "text/html" }));
                host.blobUrl = source;
            } catch (e) {
                host.status.textContent = "App payload is invalid";
                return;
            }
        }
        if (!source) {
            host.status.textContent = "App has no resource";
            return;
        }
        var frame = document.createElement("iframe");
        frame.className = "gb-app-frame";
        frame.setAttribute("sandbox", "allow-scripts");
        frame.setAttribute("title", safeTitle(payload.title));
        frame.setAttribute("referrerpolicy", "no-referrer");
        frame.dataset.origin = frameOrigin(source);
        frame.src = source;
        frame.addEventListener("load", function () {
            host.status.textContent = "Starting…";
            sendInitialize(host);
            // Most generated apps do not implement the JSON-RPC handshake;
            // "Starting…" must not be the permanent label of a working app.
            setTimeout(function () {
                if (host.status.textContent === "Starting…") {
                    host.status.textContent = "Running";
                }
            }, 1200);
        });
        host.body.appendChild(frame);
        host.frame = frame;
        live.push(host);
        enforceLiveCap();
    }

    function unloadFrame(host) {
        if (!host.frame) return;
        try { host.frame.remove(); } catch (e) {}
        host.frame = null;
        if (host.blobUrl) {
            URL.revokeObjectURL(host.blobUrl);
            host.blobUrl = null;
        }
        var index = live.indexOf(host);
        if (index >= 0) live.splice(index, 1);
        host.poster.hidden = false;
        host.status.textContent = "Paused";
    }

    function enforceLiveCap() {
        while (live.length > MAX_LIVE) {
            var oldest = live.shift();
            if (oldest && oldest.frame) unloadFrame(oldest);
        }
    }

    function rpc(id, method, params) {
        return { jsonrpc: "2.0", id: id, method: method, params: params || {} };
    }

    function notify(method, params) {
        return { jsonrpc: "2.0", method: method, params: params || {} };
    }

    function send(host, message) {
        if (!host.frame || !host.frame.contentWindow) return;
        try {
            host.frame.contentWindow.postMessage(message, "*");
        } catch (e) {
            host.status.textContent = "Bridge unavailable";
        }
    }

    function sendInitialize(host) {
        host.nextId = 1;
        send(host, rpc(host.nextId, "ui/initialize", {
            protocolVersion: "2025-06-18",
            appInfo: { name: "General Bots", version: "1" },
            appCapabilities: {
                tools: { listChanged: false },
                resources: {},
                prompts: {},
            },
            hostInfo: { name: "General Bots chat", version: "1" },
            hostCapabilities: {
                sandbox: { mode: "opaque" },
                openLinks: {},
            },
        }));
    }

    function handleUiMessage(host, message) {
        var method = message.method || "";
        var params = message.params || {};
        if (method === "ui/message") {
            var text = params.content && params.content.type === "text"
                ? params.content.text
                : params.text || "";
            if (text && typeof window.sendMessage === "function") {
                window.sendMessage(String(text).slice(0, 8000));
            }
            return true;
        }
        if (method === "ui/update-model-context") {
            contextStore[host.payload.surface_id || "app"] = params;
            window.dispatchEvent(new CustomEvent("gb-app-context", {
                detail: { surface_id: host.payload.surface_id, context: params },
            }));
            return true;
        }
        if (method === "ui/resource-teardown") {
            unloadFrame(host);
            return true;
        }
        if (method === "tools/call") {
            var name = params.name || "";
            host.status.textContent = "Waiting for approval…";
            var confirmed = window.confirm(
                "The app wants to run '" + name + "'. This will be sent to the assistant and requires your consent."
            );
            if (!confirmed) {
                host.status.textContent = "Tool call denied";
                return true;
            }
            if (typeof window.ChatSend === "object" && window.ChatSend.sendStructured) {
                window.ChatSend.sendStructured({
                    action: "app_tool",
                    surface_id: host.payload.surface_id,
                    values: { tool: name, arguments: JSON.stringify(params.arguments || {}) },
                });
            }
            host.status.textContent = "Sent to the assistant";
            return true;
        }
        return false;
    }

    function onMessage(event) {
        var host = null;
        for (var i = 0; i < live.length; i += 1) {
            if (live[i].frame && live[i].frame.contentWindow === event.source) {
                host = live[i];
                break;
            }
        }
        if (!host) return;
        var expected = host.frame.dataset.origin || "null";
        var incoming = event.origin || "null";
        if (expected !== incoming) return;
        var size = typeof event.data === "string" ? event.data.length : 0;
        if (size > MAX_MESSAGE_BYTES) return;
        var now = Date.now();
        host.hits = (host.hits || []).filter(function (time) { return now - time < 1000; });
        if (host.hits.length >= MAX_MESSAGES_PER_SECOND) return;
        host.hits.push(now);

        var message = event.data;
        if (typeof message === "string") {
            try { message = JSON.parse(message); } catch (e) { return; }
        }
        if (!message || message.jsonrpc !== "2.0") return;

        if (message.id && message.result) {
            if (message.result.hostCapabilities || message.result.appCapabilities) {
                host.status.textContent = (window.i18n && typeof window.i18n.t === "function" && window.i18n.t("chat-ready") !== "chat-ready") ? window.i18n.t("chat-ready") : "Ready";
                send(host, notify("ui/notifications/initialized", {}));
            }
            return;
        }
        if (message.method) {
            var handled = handleUiMessage(host, message);
            if (handled && message.id) {
                send(host, { jsonrpc: "2.0", id: message.id, result: {} });
            }
        }
    }

    function render(payload, rootEl) {
        var root = rootEl || document.getElementById("messages");
        if (!root || !payload) return null;
        var host = createFrameHost(payload);
        host.card._gbAppHost = host;
        root.appendChild(host.card);

        host.card.querySelector(".gb-app-btn:nth-of-type(1)").addEventListener("click", function () {
            unloadFrame(host);
            mountFrame(host);
        });
        host.card.querySelector(".gb-app-btn:nth-of-type(2)").addEventListener("click", function () {
            host.card.classList.toggle("expanded");
        });
        host.card.querySelector(".gb-app-btn:nth-of-type(3)").addEventListener("click", function () {
            host.stopped = true;
            unloadFrame(host);
            host.status.textContent = "Stopped";
            host.poster.hidden = true;
        });
        host.poster.addEventListener("click", function () {
            host.poster.hidden = true;
            mountFrame(host);
        });

        if ("IntersectionObserver" in window) {
            host.observer = new IntersectionObserver(function (entries) {
                entries.forEach(function (entry) {
                    if (entry.isIntersecting) {
                        // A freshly appended card starts below the fold and was
                        // unloaded before the transcript scrolled; it must come
                        // back when it enters the viewport, not wait for a click.
                        if (!host.stopped && !host.frame) mountFrame(host);
                        return;
                    }
                    // Do not unload in the first seconds: the card is appended
                    // before the chat scrolls to the new message.
                    if (host.frame && Date.now() - (host.mountedAt || 0) > 2500) {
                        unloadFrame(host);
                    }
                });
            }, { rootMargin: "200px" });
            host.observer.observe(host.card);
        }
        mountFrame(host);
        return host.card;
    }

    window.addEventListener("message", onMessage);

    window.AppFrame = {
        render: render,
        renderFromFrame: function (frame, rootEl) {
            var payload = frame && frame.data ? frame.data.payload || {} : {};
            return render(payload, rootEl);
        },
        liveCount: function () { return live.length; },
        context: function (surfaceId) { return contextStore[surfaceId] || null; },
    };
})();
