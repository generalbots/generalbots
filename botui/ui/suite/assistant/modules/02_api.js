"use strict";
/**
 * 02_api.js - REST and WebSocket communication.
 */

(function () {
    var A = window.GBAssistant;
    var BASE_DELAY = 1000;
    var MAX_DELAY = 15000;

    /* ------------------------------------------------------------------ */
    /*  REST helpers                                                       */
    /* ------------------------------------------------------------------ */

    function authHeaders() {
        var h = { "Content-Type": "application/json" };
        var t = A.getToken();
        if (t) { h["Authorization"] = "Bearer " + t; }
        return h;
    }

    A.apiFetch = function (path, opts) {
        opts = opts || {};
        opts.headers = Object.assign(authHeaders(), opts.headers || {});
        return fetch(A.origin + path, opts).then(function (r) {
            if (r.status === 401) { A.handle401(path); }
            return r;
        });
    };

    /* ------------------------------------------------------------------ */
    /*  Product config                                                     */
    /* ------------------------------------------------------------------ */

    A.loadProduct = async function () {
        try {
            var r = await A.apiFetch("/api/product");
            if (!r.ok) { return; }
            var data = await r.json();
            A.productConfig = data;
            A.apps = data.apps || [];
            A.compiledFeatures = data.compiled_features || [];
            return data;
        } catch (e) {
            console.warn("[assistant] product fetch failed:", e);
        }
    };

    /* ------------------------------------------------------------------ */
    /*  Auth + user profile                                                */
    /* ------------------------------------------------------------------ */

    /** Authenticate and get a NEW session (no session_id param). */
    A.authenticate = async function () {
        try {
            var r = await A.apiFetch(
                "/api/auth?new=1&bot_name=" + encodeURIComponent(A.botName)
            );
            if (!r.ok) { return null; }
            var data = await r.json();
            A.userId = data.user_id;
            A.sessionId = data.session_id;
            A.botId = data.bot_id;
            A.currentSessionId = data.session_id;
            return data;
        } catch (e) {
            console.warn("[assistant] auth failed:", e);
            return null;
        }
    };

    /** Load user profile (display name, email). */
    A.loadUserProfile = async function () {
        try {
            var r = await A.apiFetch("/api/auth/me");
            if (!r.ok) { return; }
            var data = await r.json();
            A.userName = data.display_name || data.first_name || data.username || "";
            A.userEmail = data.email || "";
        } catch (e) {
            console.warn("[assistant] user profile failed:", e);
        }
    };

    /* ------------------------------------------------------------------ */
    /*  Models                                                             */
    /* ------------------------------------------------------------------ */

    A.loadModels = async function () {
        try {
            var r = await A.apiFetch(
                "/api/chat/models?bot=" + encodeURIComponent(A.botName)
            );
            if (!r.ok) { return; }
            var data = await r.json();
            A.tiers = data.tiers || [];
            A.defaultModel = data.default_model || "";
            if (data.destination) { A.destination = data.destination; }
            try {
                A.selectedModel = localStorage.getItem("gb-model-preference");
            } catch (e) { /* ignore */ }
            /* A remembered choice is kept only while it is still on offer:
               after the model in use changes (Settings > Modelo), the old
               name must not stay on the status line nor go out with a turn. */
            var offered = A.selectedModel === data.default_model ||
                A.tiers.some(function (tier) { return tier.model === A.selectedModel; });
            if (A.selectedModel && !offered) {
                A.selectedModel = null;
                try { localStorage.removeItem("gb-model-preference"); } catch (e) { /* ignore */ }
            }
            if (!A.selectedModel && data.default_model) {
                A.selectedModel = data.default_model;
            }
            if (!A.selectedModel && A.tiers.length > 0) {
                A.selectedModel = A.tiers[0].model;
            }
        } catch (e) {
            console.warn("[assistant] models fetch failed:", e);
        }
    };

    /* ------------------------------------------------------------------ */
    /*  Sessions                                                           */
    /* ------------------------------------------------------------------ */

    A.loadSessions = async function () {
        try {
            var r = await A.apiFetch(
                "/api/chat/history/sessions?limit=50&bot_name=" +
                    encodeURIComponent(A.botName)
            );
            if (!r.ok) { return; }
            var data = await r.json();
            /* Server returns session_id, not id */
            A.sessions = (data.sessions || []).map(function (s) {
                return {
                    id: s.session_id || s.id,
                    title: s.title,
                    updated_at: s.updated_at || s.created_at
                };
            });
        } catch (e) {
            console.warn("[assistant] sessions fetch failed:", e);
        }
    };

    A.loadSessionMessages = async function (sessId) {
        try {
            var r = await A.apiFetch(
                "/api/chat/history/sessions/" +
                    encodeURIComponent(sessId) +
                    "/messages?limit=500&offset=0&include_activities=1"
            );
            if (!r.ok) { return []; }
            var data = await r.json();
            /* #2562 — stash activities so renderHistory can show step cards. */
            A._lastHistoryActivities = data.activities || [];
            return data.messages || [];
        } catch (e) {
            console.warn("[assistant] messages fetch failed:", e);
            return [];
        }
    };

    /** #2562 — Truncate history from a given message index (for regenerate/edit). */
    A.truncateHistory = async function (sessId, fromIndex) {
        try {
            var r = await A.apiFetch(
                "/api/chat/history/sessions/" +
                    encodeURIComponent(sessId) +
                    "/truncate",
                {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ from_index: fromIndex }),
                }
            );
            if (!r.ok) {
                console.warn("[assistant] truncate failed:", r.status);
                return false;
            }
            return true;
        } catch (e) {
            console.warn("[assistant] truncate error:", e);
            return false;
        }
    };

    /** Start a new conversation (call /api/auth without session_id). */
    A.newSession = async function () {
        try {
            var r = await A.apiFetch(
                "/api/auth?new=1&bot_name=" + encodeURIComponent(A.botName)
            );
            if (!r.ok) { return null; }
            var data = await r.json();
            A.sessionId = data.session_id;
            A.currentSessionId = data.session_id;
            return data.session_id;
        } catch (e) {
            console.warn("[assistant] new session failed:", e);
            return null;
        }
    };

    A.renameSession = async function (sessId, title) {
        try {
            await A.apiFetch(
                "/api/chat/history/sessions/" + encodeURIComponent(sessId),
                { method: "PATCH", body: JSON.stringify({ title: title }) }
            );
        } catch (e) {
            console.warn("[assistant] rename failed:", e);
        }
    };

    A.deleteSession = async function (sessId) {
        try {
            await A.apiFetch(
                "/api/chat/history/sessions/" + encodeURIComponent(sessId),
                { method: "DELETE" }
            );
        } catch (e) {
            console.warn("[assistant] delete failed:", e);
        }
    };

    /* ------------------------------------------------------------------ */
    /*  WebSocket                                                          */
    /* ------------------------------------------------------------------ */

    function wsUrl() {
        var loc = window.location;
        var proto = loc.protocol === "https:" ? "wss:" : "ws:";
        var host = A.origin
            ? A.origin.replace(/^https?:/, proto)
            : proto + "//" + loc.host;
        var token = A.getToken() || "";
        return (
            host + "/ws" +
            "?token=" + encodeURIComponent(token) +
            "&session_id=" + encodeURIComponent(A.currentSessionId || A.sessionId || "") +
            "&user_id=" + encodeURIComponent(A.userId || "") +
            "&bot_name=" + encodeURIComponent(A.botName)
        );
    }

    A.connectWs = function () {
        if (A.ws && A.ws.readyState <= 1) { return; }
        try { A.ws = new WebSocket(wsUrl()); }
        catch (e) { scheduleReconnect(); return; }

        A.ws.onopen = function () {
            var wasReconnect = A.reconnectAttempts > 0;
            A.wsConnected = true;
            A.reconnectAttempts = 0;
            updateStatus("connected");
            if (wasReconnect) {
                A.announce(A.t("assistant.reconnected", "Connection restored"));
            } else {
                A.announce(A.t("assistant.connected", "Connected"));
            }
        };
        A.ws.onclose = function () {
            A.wsConnected = false;
            updateStatus("disconnected");
            /* If streaming was in progress, finalize with partial content. */
            if (A.isStreaming && typeof A.endStream === "function") {
                A.endStream();
            }
            scheduleReconnect();
        };
        A.ws.onerror = function () {
            A.wsConnected = false;
            updateStatus("error");
        };
        A.ws.onmessage = function (evt) {
            try { dispatch(JSON.parse(evt.data)); } catch (e) { /* non-JSON */ }
        };
    };

    /** Reconnect WS with a new session id (after new-chat or load). */
    A.reconnectWs = function () {
        if (A.ws) {
            try { A.ws.onclose = null; A.ws.close(); } catch (e) { /* */ }
        }
        A.ws = null;
        A.wsConnected = false;
        A.connectWs();
    };

    function scheduleReconnect() {
        /* Never give up: exponential back-off 1 → 2 → 4 → 8 → 15 s cap. */
        var delay = Math.min(BASE_DELAY * Math.pow(2, Math.min(A.reconnectAttempts, 10)), MAX_DELAY);
        A.reconnectAttempts++;
        clearTimeout(A.reconnectTimer);
        A.reconnectTimer = setTimeout(function () { A.connectWs(); }, delay);
    }

    /* Reconnect immediately when the window regains focus. */
    window.addEventListener("focus", function () {
        if (!A.wsConnected && (!A.ws || A.ws.readyState > 1)) {
            clearTimeout(A.reconnectTimer);
            A.reconnectAttempts = 0;
            A.connectWs();
        }
    });
    document.addEventListener("visibilitychange", function () {
        if (!document.hidden && !A.wsConnected && (!A.ws || A.ws.readyState > 1)) {
            clearTimeout(A.reconnectTimer);
            A.reconnectAttempts = 0;
            A.connectWs();
        }
    });

    function updateStatus(state) {
        var dot = document.getElementById("gbStatusDot");
        if (dot) {
            dot.className = "gb-status-dot" +
                (state === "connected" ? " connected" : "") +
                (state === "error" ? " error" : "");
        }
        /* Reconnecting banner: shown above the composer when disconnected,
           hides automatically when the socket reconnects. */
        showReconnectBanner(state !== "connected");
    }

    var reconnectBannerEl = null;
    function showReconnectBanner(show) {
        if (show && !reconnectBannerEl) {
            reconnectBannerEl = document.createElement("div");
            reconnectBannerEl.id = "gbReconnectBanner";
            reconnectBannerEl.className = "gb-reconnect-banner";
            reconnectBannerEl.setAttribute("role", "status");
            reconnectBannerEl.innerHTML =
                '<span class="gb-reconnect-pulse" aria-hidden="true"></span>' +
                '<span>' + A.escapeHtml(A.t("assistant.reconnecting", "Reconectando…")) + '</span>';
            var dock = document.getElementById("gbDockAbove") ||
                       document.getElementById("gbComposerDock");
            if (dock) { dock.parentNode.insertBefore(reconnectBannerEl, dock); }
        }
        if (!show && reconnectBannerEl) {
            reconnectBannerEl.remove();
            reconnectBannerEl = null;
        }
    }

    /* ---- WS dispatch ---- */

    function dispatch(data) {
        /* Suggestions (can arrive on any frame) */
        if (data.suggestions && data.suggestions.length) {
            A.suggestions = data.suggestions;
            if (typeof A.onSuggestions === "function") {
                A.onSuggestions(data.suggestions);
            }
        }

        if (data.type === "connected") {
            if (data.user_id) { A.userId = data.user_id; }
            if (data.session_id) { A.currentSessionId = data.session_id; }
            return;
        }

        /* Run events (AG-UI protocol): gb_event frames.
           Activity events are routed to block renderers. */
        if (data.gb_event) {
            if (typeof A.onActivityEvent === "function") {
                A.onActivityEvent(data);
            }
            return;
        }

        /* Bot response (streaming or complete) */
        if (data.message_type === 2 || data.type === "bot_response") {
            if (typeof A.onBotMessage === "function") {
                A.onBotMessage(data);
            }
            return;
        }

        if (data.type === "message_received") {
            if (data.session_id) { A.currentSessionId = data.session_id; }
            return;
        }

        if (data.type === "chat_status") {
            if (typeof A.onChatStatus === "function") {
                A.onChatStatus(data);
            }
            return;
        }

        if (data.type === "run_cancelled") {
            A.isStreaming = false;
            if (typeof A.onStreamEnd === "function") {
                A.onStreamEnd();
            }
            return;
        }
    }

    /* ---- Send ---- */

    A.sendMessage = function (text, files, messageType) {
        if (!A.ws || A.ws.readyState !== 1) { return false; }
        var payload = {
            bot_id: A.botId || "",
            user_id: A.userId || "",
            session_id: A.currentSessionId || A.sessionId || "",
            channel: "web",
            content: text,
            message_type: messageType || 1,
            /* Agent mode is off by default; the user enables it via the
               composer toggle. The server checks the backend at startup
               and disables agent mode gracefully when unavailable. */
            agent_enabled: A.agentEnabled || false,
            ui_automation: false,
            web_search: false,
        };
        if (A.selectedModel) { payload.model_preference = A.selectedModel; }
        if (files && files.length > 0) { payload.files = files; payload.file = files[0]; }
        A.ws.send(JSON.stringify(payload));
        return true;
    };

    A.cancelGeneration = function () {
        if (!A.ws || A.ws.readyState !== 1) { return; }
        A.ws.send(JSON.stringify({
            type: "run_cancel",
            session_id: A.currentSessionId || A.sessionId || "",
        }));
        /* Optimistic cancel: restore the composer immediately.  Late frames
           from the model are suppressed by the _cancelledAt guard in
           onBotMessage so they cannot re-lock the composer (#2562). */
        A._cancelledAt = Date.now();
        A.isStreaming = false;
        A.hideThinking();
        if (typeof A.onStreamEnd === "function") { A.onStreamEnd(); }
    };
})();
