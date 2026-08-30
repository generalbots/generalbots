/**
 * Vibe Professional Dialogs (#806 rewrite) — shared framework.
 * Each dialog floats in its own VB6-style tool window (no full-screen
 * modals); content modules (vibe-dialog-db, vibe-dialog-git, etc.) register
 * builders via VibeDialogs.register. Falls back to the old in-window mask
 * when the desktop shell (WindowManager) is not present (isolated runs).
 */
(function () {
    "use strict";

    var registry = {};
    var current = null;
    var TOOL_PREFIX = "vibe-tool-";

    function authHeaders(extra) {
        var headers = Object.assign({}, extra || {});
        var token =
            (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
            localStorage.getItem("token") ||
            localStorage.getItem("id_token") ||
            localStorage.getItem("gb-access-token") ||
            sessionStorage.getItem("gb-access-token") ||
            "";
        if (token) headers["Authorization"] = "Bearer " + token;
        return headers;
    }

    function api(path, options) {
        options = options || {};
        options.headers = authHeaders(options.headers || {});
        if (options.body && typeof options.body !== "string") {
            options.headers["Content-Type"] = "application/json";
            options.body = JSON.stringify(options.body);
        }
        return fetch(path, options).then(function (resp) {
            return resp.json().catch(function () {
                return { success: false, error: "HTTP " + resp.status };
            });
        });
    }

    function esc(s) {
        var d = document.createElement("div");
        d.textContent = s == null ? "" : String(s);
        return d.innerHTML;
    }

    function el(tag, cls, text) {
        var node = document.createElement(tag);
        if (cls) node.className = cls;
        if (text != null) node.textContent = text;
        return node;
    }

    // Resolve the container element that will host the dialog content.
    // Always a floating tool window (VB6-style) — never a modal mask.
    function resolveBody(name, title) {
        var wm = window.WindowManager;
        if (wm && wm.openToolWindowBody) {
            var wmBody = wm.openToolWindowBody(TOOL_PREFIX + name, title || name, { ownerId: "vibe" });
            if (wmBody) {
                wmBody.innerHTML = "";
                var wrap = document.createElement("div");
                wrap.className = "vibe-dialog vibe-dialog-toolwindow";
                wmBody.appendChild(wrap);
                return wrap;
            }
        }
        return null;
    }

    function open(name, title) {
        // Tear down and clear any previously open dialog before building the
        // new one — otherwise switching dialogs (db → git → code) appends the
        // new content on top of the stale body.
        var prev = registry[current];
        if (prev && prev.teardown) prev.teardown();
        current = null;

        var body = resolveBody(name, title);
        var builder = registry[name];
        if (body) {
            if (!builder) {
                body.innerHTML = '<div class="vibe-empty">Unknown dialog: ' + esc(name) + "</div>";
            } else {
                builder.build(body, {
                    api: api,
                    esc: esc,
                    el: el,
                    close: close,
                });
            }
            var wm = window.WindowManager;
            if (wm) {
                // Set the window title after the body exists.
                var wnd = wm.openWindows && wm.openWindows.find(function (w) { return w.id === TOOL_PREFIX + name; });
                if (wnd) wnd.title = title || name;
                var titleEl = document.getElementById("window-" + TOOL_PREFIX + name);
                if (titleEl) {
                    var hdr = titleEl.querySelector(".window-title, .font-mono");
                    if (hdr) hdr.textContent = title || name;
                }
                wm.focusWindow(TOOL_PREFIX + name);
            }
        } else {
            // No window manager (isolated run): build into a temporary
            // floating panel appended to the document body — still not a
            // modal mask.
            var float = document.createElement("div");
            float.className = "vibe-dialog vibe-dialog-float";
            float.style.position = "fixed";
            float.style.top = "20%";
            float.style.left = "30%";
            float.style.zIndex = "9999";
            float.style.background = "var(--gb-surface, #fff)";
            float.style.border = "1px solid var(--gb-border, #ccc)";
            float.style.borderRadius = "8px";
            float.style.boxShadow = "0 8px 28px rgba(0,0,0,.25)";
            float.style.minWidth = "340px";
            float.style.maxWidth = "90vw";
            var head = document.createElement("div");
            head.style.cssText = "display:flex;justify-content:space-between;align-items:center;padding:.5rem .75rem;border-bottom:1px solid var(--gb-border,#eee);font-weight:600";
            head.innerHTML = '<span>' + esc(title || name) + "</span><button data-close style=\"border:none;background:none;cursor:pointer;font-size:1rem\">✕</button>";
            var content = document.createElement("div");
            content.style.padding = "0.75rem";
            float.appendChild(head);
            float.appendChild(content);
            document.body.appendChild(float);
            head.querySelector("[data-close]").addEventListener("click", function () { float.remove(); });
            if (!builder) {
                content.innerHTML = '<div class="vibe-empty">Unknown dialog: ' + esc(name) + "</div>";
            } else {
                builder.build(content, {
                    api: api,
                    esc: esc,
                    el: el,
                    close: function () { float.remove(); current = null; },
                });
            }
        }
        current = name;
    }

    function close() {
        var name = current;
        var builder = registry[name];
        if (builder && builder.teardown) builder.teardown();
        current = null;
        var wm = window.WindowManager;
        if (wm && wm.getWindow(TOOL_PREFIX + name)) {
            wm.close(TOOL_PREFIX + name);
            return;
        }
        // Isolated fallback: remove any floating dialog panel.
        var floats = document.querySelectorAll(".vibe-dialog-float");
        floats.forEach(function (f) { f.remove(); });
    }

    // When the user closes a dialog via the window chrome ✕, tear down the
    // registered builder so sockets/terminals are disposed, not leaked.
    document.addEventListener("gb-window-close", function (e) {
        var id = e.detail && e.detail.id;
        if (id && id.indexOf(TOOL_PREFIX) === 0) {
            var name = id.substring(TOOL_PREFIX.length);
            var builder = registry[name];
            if (builder && builder.teardown) builder.teardown();
            if (current === name) current = null;
        }
    });

    function register(name, builder) {
        registry[name] = builder;
    }

    // Standalone specialist pages (#1189): render a dialog directly into a
    // page-owned host element instead of a tool window.
    function openInPage(name, title, hostEl) {
        var prev = registry[current];
        if (prev && prev.teardown) prev.teardown();
        current = null;
        var builder = registry[name];
        if (!hostEl) return;
        hostEl.innerHTML = "";
        if (!builder) {
            hostEl.innerHTML = '<div class="vibe-empty">Unknown specialist: ' + esc(name) + "</div>";
        } else {
            builder.build(hostEl, {
                api: api,
                esc: esc,
                el: el,
                close: function () { openInPage(name, title, hostEl); },
            });
        }
        current = name;
    }

    window.VibeDialogs = {
        open: open,
        close: close,
        register: register,
        api: api,
        esc: esc,
        el: el,
        openInPage: openInPage,
    };
})();