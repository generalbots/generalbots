            // Desktop-local has no separate login host: the desktop shell
            // serves its own sign-in page next to this one. Botui-served
            // installs inject GB_LOGIN_URL before this script runs, so only
            // the standalone (botserver-served) build uses the fallback.
            if (!window.GB_LOGIN_URL) {
                // #2085 — on the Tauri asset origin a relative fallback points
                // at tauri.localhost/suite/login.html, whose /api/auth/* calls
                // hit the asset protocol ("asset not found"). Target the
                // sidecar origin instead.
                var __gbAssetOrigin =
                    window.location.protocol === "tauri:" ||
                    window.location.protocol === "asset:" ||
                    window.location.hostname === "tauri.localhost";
                // #2107 — one origin for every desktop request (login, API and
                // sandbox resources): the sidecar, never the asset protocol.
                window.__GB_SERVER_ORIGIN__ = __gbAssetOrigin ? "http://127.0.0.1:8080" : "";
                window.GB_LOGIN_URL = __gbAssetOrigin
                    ? window.__GB_SERVER_ORIGIN__ + "/suite/login.html"
                    : "/suite/login.html";
            }
            // Consume a suite session token BEFORE any other script loads.
            // security-bootstrap.js snapshots the token into memory when it
            // initialises; a capture at the end of <body> would arrive after
            // that snapshot and every isAuthenticated() call would still say
            // "anonymous". Cloud JWTs (3 segments) keep using the SSO hop.
            try {
                var __tokParams = new URLSearchParams(window.location.search);
                var __tok = __tokParams.get("token");
                if (__tok && __tok.split(".").length !== 3) {
                    localStorage.setItem("gb-access-token", __tok);
                    var __u = new URL(window.location.href);
                    __u.searchParams.delete("token");
                    __u.searchParams.delete("email");
                    __u.searchParams.delete("name");
                    window.history.replaceState({}, "", __u.pathname + __u.search + __u.hash);
                }
            } catch (e) {}
            // #1894 — the desktop hides the WebView default context menu so it
            // feels native. Debugging keeps it behind an explicit flag, never
            // always-on: ?gb_debug_context_menu=1 or the stored preference.
            try {
                var __cmParams = new URLSearchParams(window.location.search);
                if (
                    __cmParams.get("gb_debug_context_menu") === "1" ||
                    localStorage.getItem("gb-debug-context-menu") === "1"
                ) {
                    window.gbDebugContextMenu = true;
                }
            } catch (e) {}
            // #2085 — the splash window must not run the shell auth gate.
            window.__GB_IS_SPLASH__ = /(^|\/)splash\.html$/.test(window.location.pathname);
            window.__BOT_IS_PUBLIC__ = false;
            // Hide the shell from the very first paint: a private bot must
            // never flash an empty desktop before the auth gate resolves.
            document.documentElement.classList.add('gb-auth-gate');
            setTimeout(function() {
                document.body.style.pointerEvents = 'auto';
                document.body.style.overflow = 'auto';
            }, 8000);
        
