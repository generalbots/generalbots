            // Desktop-local has no separate login host: the desktop shell
            // serves its own sign-in page next to this one. Botui-served
            // installs inject GB_LOGIN_URL before this script runs, so only
            // the standalone (botserver-served) build uses the fallback.
            if (!window.GB_LOGIN_URL) {
                window.GB_LOGIN_URL = "/suite/login.html";
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
            window.__BOT_IS_PUBLIC__ = false;
            // #1806 — Apply global density preferences BEFORE first paint so
            // the desktop shell never flashes in normal density then snaps to
            // compact.  Read straight from localStorage (synchronous, no
            // flicker).
            try {
                if (localStorage.getItem('gb-compact-mode') === 'true') {
                    document.documentElement.classList.add('compact-mode');
                }
                if (localStorage.getItem('gb-animations') === 'false') {
                    document.documentElement.classList.add('no-animations');
                }
            } catch (e) {}
            // Hide the shell from the very first paint: a private bot must
            // never flash an empty desktop before the auth gate resolves.
            document.documentElement.classList.add('gb-auth-gate');
            setTimeout(function() {
                document.body.style.pointerEvents = 'auto';
                document.body.style.overflow = 'auto';
            }, 8000);
        
