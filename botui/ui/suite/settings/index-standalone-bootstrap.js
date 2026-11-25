
    // Settings is also opened as a full page (admin deep link), outside the
    // desktop shell; there it needs the local htmx and the security bootstrap
    // that adds the Bearer token to HTMX/fetch/XHR requests (#1821/#1839).
    // This used the legacy writer twice, at two cache-busters ("?v=6" and
    // "?v=7"), so which bootstrap version ran depended on parse timing (#2332).
    // Each file is now appended once, in order, and only when missing.
    (function () {
        if (window.top !== window.self) return; // inside the shell: already loaded
        var files = [];
        if (!window.htmx) files.push("/suite/js/vendor/htmx.min.js");
        if (!window.GBSecurity) {
            files.push(
                "/suite/js/security-bootstrap/01_tokens.js?v=7",
                "/suite/js/security-bootstrap/02_interceptors.js?v=7",
                "/suite/js/security-bootstrap/99_init.js?v=7"
            );
        }
        // #2334 — global HTMX error UX must exist on standalone pages too.
        if (!window.GBToasts) files.push("/suite/js/gb-toasts.js?v=3");
        if (!window.__GB_HTMX_ERRORS_INSTALLED__) files.push("/suite/js/base/03_htmx_errors.js?v=3");
        (function loadNext(index) {
            if (index >= files.length) return;
            var script = document.createElement("script");
            script.src = files[index];
            script.onload = function () { loadNext(index + 1); };
            script.onerror = function () {
                console.error("settings: failed to load " + files[index]);
                loadNext(index + 1);
            };
            document.head.appendChild(script);
        })(0);
    })();
