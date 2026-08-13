/**
 * Vibe Professional Dialogs (#806 rewrite) — shared framework.
 * One mask renders each dialog; content modules (vibe-dialog-db,
 * vibe-dialog-git, etc.) register builders via VibeDialogs.register.
 */
(function () {
    "use strict";

    var registry = {};
    var current = null;

    function authHeaders(extra) {
        var headers = Object.assign({}, extra || {});
        var token =
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

    function open(name, title) {
        var mask = document.getElementById("vibeDialogMask");
        if (!mask) return;
        var body = document.getElementById("vibeDialogBody");
        var titleEl = document.getElementById("vibeDialogTitle");
        var builder = registry[name];
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
        if (titleEl && title) {
            titleEl.innerHTML = '<span class="dot"></span>' + esc(title);
        }
        current = name;
        mask.classList.add("open");
    }

    function close() {
        var mask = document.getElementById("vibeDialogMask");
        if (!mask) return;
        var body = document.getElementById("vibeDialogBody");
        var builder = registry[current];
        if (builder && builder.teardown) builder.teardown();
        body.innerHTML = "";
        current = null;
        mask.classList.remove("open");
    }

    function register(name, builder) {
        registry[name] = builder;
    }

    window.VibeDialogs = {
        open: open,
        close: close,
        register: register,
        api: api,
        esc: esc,
        el: el,
    };
})();