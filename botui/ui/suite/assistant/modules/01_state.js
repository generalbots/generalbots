"use strict";
/**
 * 01_state.js - Shared state and i18n for the assistant shell.
 */

window.GBAssistant = window.GBAssistant || {};

(function () {
    var A = window.GBAssistant;

    /* ---- Connection ---- */
    A.ws = null;
    A.wsConnected = false;
    A.reconnectAttempts = 0;
    A.reconnectTimer = null;

    /* ---- Identity ---- */
    A.userId = null;
    A.sessionId = null;
    A.botName = "default";
    A.botId = null;
    A.userName = null;
    A.userEmail = null;

    /* ---- Product config ---- */
    A.productConfig = null;
    A.compiledFeatures = [];
    A.apps = [];

    /* ---- Chat ---- */
    A.currentSessionId = null;
    A.sessions = [];
    A.isStreaming = false;
    A.streamingContent = "";
    A.streamingReasoning = "";
    A.streamingEl = null;
    A.lastCompletedContent = "";
    A.lastCompletedTime = 0;
    A.pendingFiles = [];

    /* ---- Models ---- */
    A.tiers = [];
    A.defaultModel = null;
    A.selectedModel = null;
    A.destination = null;

    /* ---- Suggestions ---- */
    A.suggestions = [];

    /* ---- UI state ---- */
    A.currentView = "chat";
    A.sidebarCollapsed = false;

    /* ---- i18n ---- */
    A.locale = "pt-BR";

    A.initLocale = function () {
        var stored = null;
        try { stored = localStorage.getItem("gb-locale"); } catch (e) { /* */ }
        if (!stored) {
            try { stored = localStorage.getItem("gb-language"); } catch (e) { /* */ }
        }
        if (stored) {
            A.locale = stored;
        } else {
            var nav = (navigator.language || navigator.userLanguage || "pt-BR");
            A.locale = nav.toLowerCase().startsWith("pt") ? "pt-BR" : nav;
        }
        var dicts = window.GBAssistantLocales || {};
        if (!dicts[A.locale]) {
            var lang = A.locale.split("-")[0];
            if (dicts[lang]) { A.locale = lang; }
            else { A.locale = "pt-BR"; }
        }
    };

    A.t = function (key, fallback) {
        var dicts = window.GBAssistantLocales || {};
        var dict = dicts[A.locale] || dicts["pt-BR"] || dicts["en"] || {};
        return dict[key] || fallback || key;
    };

    /* ---- Helpers ---- */

    A.getToken = function () {
        if (window.GBSecurity && window.GBSecurity.getToken) {
            return window.GBSecurity.getToken();
        }
        try {
            return (
                localStorage.getItem("gb-access-token") ||
                localStorage.getItem("management_token") ||
                sessionStorage.getItem("gb-access-token") ||
                null
            );
        } catch (e) {
            return null;
        }
    };

    A.origin = window.__GB_SERVER_ORIGIN__ || "";

    A.announce = function (text) {
        var el = document.getElementById("gbSrLive");
        if (el) {
            el.textContent = "";
            setTimeout(function () { el.textContent = text; }, 60);
        }
    };

    A.loginUrl = function () {
        return window.GB_LOGIN_URL || A.origin + "/suite/login.html";
    };

    A.goToLogin = function (redirect) {
        var url = A.loginUrl();
        if (redirect) {
            url += (url.indexOf("?") >= 0 ? "&" : "?") +
                   "redirect=" + encodeURIComponent(redirect);
        }
        window.location.href = url;
    };

    A.signOut = function () {
        try { sessionStorage.setItem("gb-signed-out", "true"); } catch (e) { /* */ }
        try {
            fetch(A.origin + "/api/auth/logout", {
                method: "POST",
                headers: { "Authorization": "Bearer " + (A.getToken() || "") }
            }).catch(function () { /* ignore */ });
        } catch (e) { /* */ }
        var keys = [
            "gb-access-token", "management_token", "management_email",
            "management_name", "management_is_admin", "gb_selected_bot",
            "gb-user-data"
        ];
        for (var i = 0; i < keys.length; i++) {
            try { localStorage.removeItem(keys[i]); } catch (e) { /* */ }
        }
        try {
            Object.keys(localStorage).forEach(function (k) {
                if (k.indexOf("gb_chat_") === 0) {
                    localStorage.removeItem(k);
                }
            });
        } catch (e) { /* */ }
        try { sessionStorage.removeItem("gb-access-token"); } catch (e) { /* */ }
        A.goToLogin();
    };

    A.handle401 = function (url) {
        if (url && url.indexOf("/api/i18n/") !== -1) { return; }
        /* Auth-critical endpoints: a 401 from these means the token
           itself is invalid (expired or tampered), not a scope issue.
           Clear the token and redirect to sign-in. */
        var isCritical = url && (
            url.indexOf("/api/auth") === 0 ||
            url.indexOf("/api/chat/history") === 0
        );
        if (isCritical) {
            var keys = ["gb-access-token", "management_token"];
            for (var i = 0; i < keys.length; i++) {
                try { localStorage.removeItem(keys[i]); } catch (e) { /* */ }
            }
            try { sessionStorage.removeItem("gb-access-token"); } catch (e) { /* */ }
            try { sessionStorage.setItem("gb-session-expired", "1"); } catch (e) { /* */ }
            A.goToLogin(window.location.href);
            return;
        }
        /* Background 401s while the user holds a valid token are NOT
           session expiry: the endpoint rejected it for its own reasons
           (scope, feature gate, stale session id). Redirecting on these
           was the root cause of conversations failing to load. */
        if (A.getToken()) {
            console.warn("[assistant] 401 from " + (url || "unknown") + " (token present, not redirecting)");
            return;
        }
        A.goToLogin(window.location.href);
    };
})();
