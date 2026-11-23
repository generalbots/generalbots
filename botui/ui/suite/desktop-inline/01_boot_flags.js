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
            // #2812 — one-time, expiring state for the suite SSO hop. The
            // value lives in sessionStorage on this origin, never in a URL;
            // a replayed hop cannot recreate a consumed state.
            window.GB_SSO_STATE = (function () {
                var KEY = "gb_sso_state";
                function create() {
                    var state = (window.crypto && crypto.getRandomValues)
                        ? Array.prototype.map.call(crypto.getRandomValues(new Uint8Array(12)), function (x) { return ("0" + x.toString(16)).slice(-2); }).join("")
                        : String(Date.now()) + "-" + String(Math.random()).slice(2);
                    var ts = Date.now();
                    try { sessionStorage.setItem(KEY, state + "." + ts); } catch (e) {}
                    return { state: state, ts: ts };
                }
                function consume(state, ts) {
                    var raw = null;
                    try { raw = sessionStorage.getItem(KEY); sessionStorage.removeItem(KEY); } catch (e) {}
                    if (!raw || !state || !/^\d{10,15}$/.test(String(ts))) return false;
                    var t = Number(ts), now = Date.now();
                    if (now - t > 60000 || t - now > 5000) return false;
                    var p = raw.split(".");
                    return p.length === 2 && p[0] === state && p[1] === String(ts);
                }
                function fromUrl() {
                    var q = new URLSearchParams(window.location.search);
                    var f = new URLSearchParams(window.location.hash.replace(/^#/, ""));
                    return { state: f.get("gb_state") || q.get("gb_state") || q.get("state"),
                             ts: f.get("gb_ts") || q.get("gb_ts") || q.get("ts") };
                }
                return { create: create, consume: consume, fromUrl: fromUrl };
            })();
            // A returning SSO hop must present an unused state; the token the
            // handoff page stored is discarded when it does not (#2812).
            try {
                var __rp = new URLSearchParams(window.location.search);
                if (__rp.has("state") || __rp.has("gb_state")) {
                    var __rs = window.GB_SSO_STATE.fromUrl();
                    if (!window.GB_SSO_STATE.consume(__rs.state, __rs.ts)) {
                        localStorage.removeItem("gb-access-token");
                        sessionStorage.removeItem("gb-access-token");
                    }
                    var __ru = new URL(window.location.href);
                    ["state", "ts", "gb_state", "gb_ts"].forEach(function (k) { __rp.delete(k); });
                    __ru.search = __rp.toString();
                    window.history.replaceState({}, "", __ru.pathname + __ru.search + __ru.hash);
                }
            } catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); }
            // Consume a suite session token BEFORE any other script loads.
            // security-bootstrap.js snapshots the token into memory when it
            // initialises; a capture at the end of <body> would arrive after
            // that snapshot and every isAuthenticated() call would still say
            // "anonymous". Cloud JWTs (3 segments) keep using the SSO hop.
            try {
                var __frag = new URLSearchParams(window.location.hash.replace(/^#/, ""));
                var __tokParams = new URLSearchParams(window.location.search);
                var __tok = __frag.get("gb_token") || __frag.get("token") || __tokParams.get("token");
                var __st = window.GB_SSO_STATE.fromUrl();
                if (__tok && __tok.split(".").length !== 3 &&
                    (!__st.state || window.GB_SSO_STATE.consume(__st.state, __st.ts))) {
                    localStorage.setItem("gb-access-token", __tok);
                    var __u = new URL(window.location.href);
                    ["token", "email", "name", "gb_token", "gb_state", "gb_ts"].forEach(function (k) { __u.searchParams.delete(k); });
                    __u.hash = "";
                    window.history.replaceState({}, "", __u.pathname + __u.search);
                }
            } catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); }
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
            } catch (e) { window.gbLogError ? gbLogError(e, "js") : console.error("js", e); }
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
        
