/* Vibe Shell toolbar — split from vibe-shell/10_toolbar.js for #1731 */

"use strict";
/**
 * Vibe Shell — single-row command bar (VB4-inspired style only).
 * One compact row: Project + Branch selectors, transport (Run/Hide/Stop),
 * the shared desktop apps (Terminal, Browser, Chat), the Vibe tool windows
 * (Runner Log, Knowledge Graph, Canvas, Metrics) and New Project.
 *
 * Style follows VB4's toolbar: one row of big buttons, each an SVG icon
 * over a short label. No status bar, no menu bar — Run/Hide/Stop light up
 * instead of a status readout; feedback is a transient hint pill.
 */

    var S = window.VibeShell;

    function wm() {
        return typeof window.WindowManager !== "undefined" ? window.WindowManager : null;
    }

    function toolbarEl(tag, cls, text) {
        var node = document.createElement(tag);
        if (cls) node.className = cls;
        if (text != null) node.textContent = text;
        return node;
    }

    /* Open one of the shared desktop apps (they also run standalone from the
       desktop shell). Deep-link params e.g. pick the project VM (terminal) or
       load a URL (browser). */
    function openSharedApp(appId, params) {
        var mgr = wm();
        if (mgr && mgr.getApp && mgr.getApp(appId) && typeof mgr.openDeepLink === "function") {
            mgr.openDeepLink(appId, params || {}, { ownerId: "vibe" });
            return;
        }
        if (mgr && mgr.getApp && mgr.getApp(appId) && typeof mgr.launchFromMenu === "function") {
            var app = mgr.getApp(appId);
            mgr.launchFromMenu(appId, app.title, app.hxGet);
            return;
        }
        var app = (window.APPS_REGISTRY || []).find(function (a) { return a.id === appId; });
        if (app) {
            // Standalone fallback (no desktop WindowManager): app partials
            // are HTMX fragments — a raw _blank open renders an
            // unbootstrapped empty shell. Route through the desktop shell
            // (/suite/desktop.html#/{app}) so the fragment boots inside the
            // real desktop; params travel via the query string (browser.html
            // deepLinkUrl reads ?url=...).
            var params2 = params || {};
            var qs = Object.keys(params2).map(function (k) {
                return encodeURIComponent(k) + "=" + encodeURIComponent(String(params2[k]));
            }).join("&");
            var target = "/suite/desktop.html?app=" + encodeURIComponent(appId) + (qs ? "&" + qs : "");
            window.open(target, "_blank", "noopener");
        }
    }

    /* ── Shared app openers ────────────────────────────────────────── */
    function openTerminal() {
        var pid = S.projectId();
        openSharedApp("terminal", pid ? { project: pid } : {});
    }

    /* #1192 — run the project's OWN custom app: the LLM writes source files to
       the project workspace (`VIBE_WORKSPACE_ROOT/{slug}/`). If it contains an
       `index.html`, Play serves that app directly through the authenticated
       workspace route; otherwise it falls back to a deployed preview URL. */
    function workspaceServeUrl(projectId) {
        if (!projectId) return Promise.reject(new Error("Select a project first"));
        return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId) + "/files")
            .then(function (r) { return r.json(); })
            .then(function (data) {
                var files = (data && data.files) || [];
                var hasIndex = files.some(function (f) { return f === "index.html"; });
                if (!hasIndex) return null;
                var token = (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) || localStorage.getItem("token") || localStorage.getItem("id_token") || localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
                var base = window.location.origin + "/api/vibe/projects/" + encodeURIComponent(projectId) + "/serve/index.html";
                return token ? base + "?token=" + encodeURIComponent(token) : base;
            })
            .catch(function () { return null; });
    }

    function resolvePreviewUrl(projectId) {
        if (!projectId) return Promise.reject(new Error("Select a project first"));
        return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId))
            .then(function (r) { return r.json(); })
            .then(function (projectData) {
                if (projectData && projectData.success === false) throw new Error(projectData.error || "Project lookup failed");
                var project = projectData && projectData.project;
                var env = (project && (project.environment || project.env)) || "development";
                return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId) + "/preview?env=" + encodeURIComponent(env));
            })
            .then(function (result) {
                if (typeof result === "string") return result;
                return result.json();
            })
            .then(function (data) {
                if (typeof data === "string") return data;
                var payload = data && data.data ? data.data : data;
                var url = payload && payload.preview_url;
                if (!url || !/^https?:/i.test(String(url))) throw new Error("No live preview — deploy the project first");
                return String(url);
            });
    }

    function openBrowser(url) {
        if (url) {
            // The browser window renders the preview inside an iframe, which
            // cannot send an Authorization header — the vm-preview/serve
            // endpoints accept `?token=` instead. Resolve the token here (the
            // shell knows it via getGBAccessToken()) and append it to the URL
            // so the app is authenticated regardless of which browser.html
            // version is cached in the service worker.
            url = withPreviewToken(url);
            openSharedApp("browser", { url: url });
            return;
        }
        openSharedApp("browser", {});
    }

    // Append the caller's access token to a same-origin preview URL so the
    // embedded iframe can authenticate. No-op for foreign/external URLs.
    function withPreviewToken(url) {
        try {
            var u = new URL(url, window.location.href);
            if (u.origin !== window.location.origin) return url;
            var path = u.pathname;
            var isPreview = /^\/api\/vibe\/projects\/[^\/]+\/(vm-preview|serve)/.test(path);
            if (!isPreview || (u.search || "").indexOf("token=") !== -1) return url;
            var tok = window.getGBAccessToken ? window.getGBAccessToken() : null;
            if (!tok) {
                tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") ||
                    localStorage.getItem("management_token") || localStorage.getItem("token") || localStorage.getItem("gb_token") || "";
            }
            if (!tok) return url;
            u.search = (u.search ? u.search + "&" : "?") + "token=" + encodeURIComponent(tok);
            return u.toString();
        } catch (e) {
            return url;
        }
    }

    /* #1271 — open a project's app in the Browser window. Run on the dev VM
       first (real node process); fall back to the static workspace stream,
       then to a deployed preview URL. Used by the toolbar Browser/Run
       buttons AND by the chat flow: a chat message that changes the app
       should end with the browser showing the result on current dev. */
    function openProjectApp(projectId) {
        if (!projectId) {
            openBrowser(null);
            return Promise.resolve();
        }
        return startDevVm(projectId)
            .then(function (vm) {
                openBrowser(vm.url);
                setRunVisual(true);
                flashHint("RUNNING ON THE DEV VM — OPENING BROWSER");
            })
            .catch(function () {
                return workspaceServeUrl(projectId)
                    .then(function (url) {
                        if (url) return url;
                        return resolvePreviewUrl(projectId);
                    })
                    .then(function (url) {
                        openBrowser(url);
                        setRunVisual(true);
                        flashHint("OPENED STATIC PREVIEW (NO VM)");
                    })
                    .catch(function () {
                        openBrowser(null);
                        flashHint("NO LIVE PREVIEW — DEPLOY TO SEE YOUR APP");
                    });
            });
    }
