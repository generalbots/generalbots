"use strict";
/**
 * 99_init.js - Bootstrap the assistant shell.
 */

(function () {
    var A = window.GBAssistant;

    /* The desktop shell closes its splash when this fragment appears. It is
       stamped a few seconds after load as well, so a boot that stops early
       never leaves the splash in front of the window. */
    function signalBootReady() {
        if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
            try { window.history.replaceState(null, "", "#gb-boot-ready"); }
            catch (e) { /* history unavailable */ }
        }
    }
    window.addEventListener("load", function () {
        window.setTimeout(signalBootReady, 6000);
    });

    document.addEventListener("DOMContentLoaded", async function () {
        /* 1. Locale (before any label is rendered) */
        A.initLocale();

        /* 2. Translate static data-i18n elements */
        translatePage();

        /* 3. Product config + accent + pattern */
        await A.loadProduct();
        if (typeof A.restoreAccent === "function") { A.restoreAccent(); }
        if (typeof A.bootPattern === "function") { A.bootPattern(); }

        /* 4. Bot name from URL */
        var pathParts = window.location.pathname.split("/");
        var pathBot = pathParts[1] || "";
        if (pathBot === "suite" || pathBot === "auth" || pathBot === "") {
            pathBot = "default";
        }
        A.botName = window.__INITIAL_BOT_NAME__ || pathBot;

        /* 5. Auth gate */
        var noSecurity = A.productConfig && A.productConfig.no_security === true;
        if (!noSecurity) {
            var token = A.getToken();
            if (!token) { A.goToLogin(window.location.href); return; }
        }

        /* 6. Authenticate (new session -- no session_id) */
        await A.authenticate();

        /* 7. User profile */
        await A.loadUserProfile();

        /* 8. Models */
        await A.loadModels();

        /* 9. Render static UI */
        A.renderWordmark();
        A.renderUser();
        A.renderGreeting();
        A.renderModelPickers();
        A.renderStatus();
        A.filterFeatures();

        /* 10. Bind events */
        A.initComposer();
        A.initSidebarToggle();
        A.initSearch();
        A.initNavigation();
        A.initUserMenu();
        if (typeof A.initScroll === "function") { A.initScroll(); }
        if (typeof A.initExportActions === "function") { A.initExportActions(); }

        /* 11. Connect WebSocket (uses the new session) */
        A.connectWs();

        /* 12. Load conversation list (sidebar) */
        await A.loadSessions();
        if (typeof A.refreshSidebar === "function") { A.refreshSidebar(); }

        /* 13. Show suggestions from the WS (start.bas sends them on connect) */
        /* They arrive via WS frame and are rendered by onSuggestions */

        /* 14. Show empty state (new conversation) */
        A.showEmptyState();
        A.renderGreeting();
        A.onSuggestions(A.suggestions);

        /* 15. Focus the input */
        var inputEmpty = document.getElementById("gbInputEmpty");
        if (inputEmpty) { inputEmpty.focus(); }

        /* 16. Tauri boot-ready signal */
        if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
            try { window.history.replaceState(null, "", "#gb-boot-ready"); }
            catch (e) { /* */ }
        }
    });

    /** Translate data-i18n-* attributes on the page.
     *  Exposed as A.translatePage so the language switch can re-translate live. */
    A.translatePage = translatePage;
    function translatePage() {
        var els = document.querySelectorAll("[data-i18n]");
        for (var i = 0; i < els.length; i++) {
            var key = els[i].getAttribute("data-i18n");
            var val = A.t(key, null);
            if (val && val !== key) { els[i].textContent = val; }
        }
        var phs = document.querySelectorAll("[data-i18n-placeholder]");
        for (var j = 0; j < phs.length; j++) {
            var pk = phs[j].getAttribute("data-i18n-placeholder");
            var pv = A.t(pk, null);
            if (pv && pv !== pk) { phs[j].placeholder = pv; }
        }
        /* aria-label and title from data-i18n-aria / data-i18n-title */
        var arias = document.querySelectorAll("[data-i18n-aria]");
        for (var a = 0; a < arias.length; a++) {
            var ak = arias[a].getAttribute("data-i18n-aria");
            var av = A.t(ak, null);
            if (av && av !== ak) { arias[a].setAttribute("aria-label", av); }
        }
        var titles = document.querySelectorAll("[data-i18n-title]");
        for (var ti = 0; ti < titles.length; ti++) {
            var tk = titles[ti].getAttribute("data-i18n-title");
            var tv = A.t(tk, null);
            if (tv && tv !== tk) { titles[ti].setAttribute("title", tv); }
        }
    }
})();
