            // Check for fullscreen mode and auth token (from cloud dashboard)
            (function() {
                var urlParams = new URLSearchParams(window.location.search);
                if (urlParams.get('fullscreen') === 'true') {
                    document.body.classList.add('fullscreen');
                }
                var cloudToken = urlParams.get('token');
                if (cloudToken) {
                    var u = new URL(window.location.href);
                    u.searchParams.delete('token');
                    u.searchParams.delete('email');
                    u.searchParams.delete('name');
                    // Only cloud JWTs go through the SSO hop. A suite session
                    // token (`gb_...`) is already valid for /api and is stored
                    // locally; sending it to suite-sso fails with 400 because
                    // that endpoint verifies a JWT signature.
                    if (cloudToken.split('.').length === 3) {
                        window.location.replace('/api/auth/suite-sso?token=' + encodeURIComponent(cloudToken) + '&redirect=' + encodeURIComponent(u.pathname + u.search));
                        return;
                    }
                    try {
                        localStorage.setItem('gb-access-token', cloudToken);
                    } catch (e) {}
                    window.history.replaceState({}, '', u.pathname + u.search);
                }
            })();


            document.addEventListener("DOMContentLoaded", async () => {
                // Initialize WindowManager
                if (typeof window.WindowManager !== "undefined") {
                    window.wm = window.WindowManager;
                } else {
                    console.error(
                        "WindowManager class not loaded from window-manager.js",
                    );
                }

                // AUTO-OPEN an app on load: ?app=vibe (legacy ?vibe=1) opens
                // the Vibe workbench immediately, so the browser always lands
                // with the app loaded instead of a bare desktop. Waits for the
                // catalog merge when it is still in flight so deep links
                // resolve against the full registry.
                var autoApp =
                    new URLSearchParams(window.location.search).get("app") ||
                    (new URLSearchParams(window.location.search).get("vibe")
                        ? "vibe"
                        : "");
                // Respect a remembered close of the Vibe bar: closing it sets
                // gb.vibe.closed=1, so a reload of /vibe (or ?app=vibe) must
                // NOT auto-reopen it. Reopening the bar clears the flag.
                if (autoApp === "vibe") {
                    try {
                        if (localStorage.getItem("gb.vibe.closed") === "1") {
                            autoApp = "";
                        }
                    } catch (e) {
                        /* storage unavailable — keep the default auto-open */
                    }
                }
                if (autoApp && typeof window.openDeepLink === "function") {
                    var launched = false;
                    var launch = function () {
                        if (launched) return;
                        launched = true;
                        window.openDeepLink(autoApp, {});
                    };
                    if (window.__gbAppsCatalogLoaded) {
                        launch();
                    } else {
                        var timer = setTimeout(launch, 4000);
                        window.addEventListener(
                            "gb-apps-catalog-loaded",
                            function () {
                                clearTimeout(timer);
                                launch();
                            },
                            { once: true }
                        );
                    }
                }

                // Initialize ThemeManager
                if (typeof window.ThemeManager !== "undefined") {
                    window.ThemeManager.init();
                }

                // Load product config for app filtering (sidebar visibility
                // is controlled by bot public status, not product config).
                try {
                    const resp = await fetch("/api/product");
                    if (resp.ok) {
                        window.productConfig = await resp.json();
                        // #1433 — honor the .product sidebar switch: sidebar=false
                        // must actually hide the rail on boot (the Settings
                        // toggle shares the same body.sidebar-hidden class).
                        if (window.productConfig && window.productConfig.sidebar === false) {
                            document.body.classList.add("sidebar-hidden");
                        }
                        // Desktop editions disable the omnibox: the taskbar
                        // search must disappear with it, otherwise it reaches
                        // domains this product does not ship.
                        if (window.productConfig && window.productConfig.search_enabled === false) {
                            var searchWrap = document.getElementById("taskbarSearchWrap");
                            if (searchWrap) { searchWrap.remove(); }
                        }
                    }
                } catch (e) {
                    console.warn("Failed to load product config:", e);
                }

                // No-security mode: skip the entire auth gate when the
                // product is configured without authentication (desktop-local).
                var noSecurity = window.productConfig && window.productConfig.no_security === true;
                if (noSecurity) {
                    window.__BOT_IS_PUBLIC__ = true;
                    document.documentElement.classList.remove('gb-auth-gate');
                }

                // Get bot name from URL path
                const pathParts = window.location.pathname.split('/');
                const pathName = pathParts[1] || '';
                const routeName = pathName === 'sources' ? 'integrations' : pathName;
                let routeApp = (window.APPS_REGISTRY || []).find(
                    (app) => app.id === routeName,
                ) || null;
                // The packaged desktop serves the shell at /suite/desktop.html;
                // "suite" is a static segment, not a bot. Without this guard the
                // Drive app took it as the bot name and probed suite.gborg /
                // suite.gbai, so its tabs answered 'Access denied for this
                // bucket' (#1709). Assign the result so every app reads the same
                // identity (it used to be undefined here).
                const staticShell = pathName === 'suite' || pathName === 'auth';
                const botName = window.__INITIAL_BOT_NAME__ ||
                    (routeApp || staticShell ? 'default' : routeName) || 'default';
                window.__INITIAL_BOT_NAME__ = botName;

                // AUTH GATE: a private bot (public=false) must NEVER render
                // the desktop for unauthenticated visitors — not even an empty
                // shell. The shell stays hidden until this gate resolves; on
                // private+unauthenticated we redirect to login immediately.
                var authDenied = false;
                // #2085 — the splash window (asset origin) must never run the
                // auth gate: it has no login form, so the redirect opened a
                // stray tauri.localhost/suite/login.html window whose /api
                // calls hit the asset protocol ("asset not found").
                var isSplashPage = /(^|\/)splash\.html$/.test(window.location.pathname);
                if (noSecurity || isSplashPage) { /* skip auth gate entirely */ } else try {

                    if (botName && botName !== 'default') {
                        const resp = await fetch('/api/bot/public?bot_name=' + encodeURIComponent(botName) + '&_=' + Date.now());
                        if (resp.ok) {
                            const config = await resp.json();
                            const isPub = config.is_public === 'true' || config.is_public === true;
                            if (isPub) {
                                window.__BOT_IS_PUBLIC__ = true;
                                // Hide the management sidebar only for anonymous
                                // visitors; authenticated owners keep it (thin/large).
                                var isAuth = false;
                                try {
                                    isAuth = !!(window.getGBAccessToken && window.getGBAccessToken());
                                } catch (e) {}
                                if (!isAuth) {
                                    try {
                                        isAuth = !!(localStorage.getItem("gb-access-token") ||
                                            sessionStorage.getItem("gb-access-token") ||
                                            localStorage.getItem("management_token"));
                                    } catch (e) {}
                                }
                                // Login redirects back with ?token=... before any
                                // storage exists yet — that counts as authenticated.
                                if (!isAuth) {
                                    try {
                                        isAuth = new URLSearchParams(window.location.search).has("token");
                                    } catch (e) {}
                                }
                                const sidebar = document.getElementById('chatSidebar');
                                if (sidebar && !isAuth) {
                                    // Public bot + anonymous visitor: KEEP the sidebar
                                    // visible (it hosts the Sign in affordance, theme and
                                    // new-chat entry point) and hide only management
                                    // sections via the .anon-limited CSS rules.
                                    sidebar.classList.add('anon-limited');
                                }
                            } else {
                                window.__BOT_IS_PUBLIC__ = false;
                                var isAuthPriv = !!(window.getGBAccessToken && window.getGBAccessToken());
                                if (!isAuthPriv) {
                                    try {
                                        isAuthPriv = !!(localStorage.getItem("gb-access-token") ||
                                            sessionStorage.getItem("gb-access-token") ||
                                            localStorage.getItem("management_token")) ||
                                            new URLSearchParams(window.location.search).has("token");
                                    } catch (e) {}
                                }
                                if (!isAuthPriv) {
                                    authDenied = true;
                                    window.location.href = (window.GB_LOGIN_URL || '/login') + '?redirect=' + encodeURIComponent(window.location.href);
                                }
                            }
                        } else {
                            // Cannot verify public status → fail closed: the
                            // desktop must not render for unverified bots.
                            console.warn('Bot public-status check failed (HTTP ' + resp.status + ') — requiring authentication');
                            authDenied = true;
                            window.location.href = (window.GB_LOGIN_URL || '/login') + '?redirect=' + encodeURIComponent(window.location.href);
                        }
                    }
                } catch (e) {
                    console.warn('Failed to check bot public status:', e);
                    // Fail closed on network errors too — never expose the
                    // desktop of an unverifiable bot address.
                    authDenied = true;
                    window.location.href = (window.GB_LOGIN_URL || '/login') + '?redirect=' + encodeURIComponent(window.location.href);
                }
                if (authDenied) {
                    return; // keep shell hidden; browser navigates to login
                }
                document.documentElement.classList.remove('gb-auth-gate');
                
                // Open a routed suite app inside the desktop shell; bot routes
                // continue to open Chat as before.
                // Remembered close: if the routed app is the Vibe bar and the
                // user closed it earlier (gb.vibe.closed=1), do NOT auto-open
                // it here either — the URL-path route must respect the same
                // memory as the ?app= / ?vibe= query route (window-manager.js
                // records the close, opening the bar again clears it).
                if (routeApp && routeApp.id === "vibe") {
                    try {
                        if (localStorage.getItem("gb.vibe.closed") === "1") {
                            routeApp = null;
                        }
                    } catch (e) {
                        /* storage unavailable — keep the default auto-open */
                    }
                }
                if (window.wm) {
                    try {
                        // Use suite_default_app from product config, falling back to Chat.
                        var defaultAppId = (window.productConfig && window.productConfig.suite_default_app) || "chat";
                        var defaultEntry = (window.APPS_REGISTRY || []).find(function(a) { return a.id === defaultAppId; });
                        if (!defaultEntry) {
                            defaultEntry = {
                                id: defaultAppId,
                                title: defaultAppId.charAt(0).toUpperCase() + defaultAppId.slice(1),
                                hxGet: "/suite/partials/" + defaultAppId + ".html",
                            };
                        }
                        const initialApp = routeApp || defaultEntry;
                        const response = await fetch(initialApp.hxGet);
                        if (response.ok) {
                            const htmlContent = await response.text();
                            window.wm.open(
                                initialApp.id,
                                initialApp.title,
                                htmlContent,
                            );
                        }
                        // #1830 — boot is complete: the shell is mounted and
                        // the default app is open. Tell the native splash
                        // (it polls the suite URL for this fragment — no Tauri
                        // IPC exists for the suite origin) so it closes only
                        // once nothing is left to load, then drop the in-page
                        // boot splash too.
                        if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
                            try {
                                window.history.replaceState(null, "", "#gb-boot-ready");
                            } catch (hashError) { /* keep the current URL */ }
                        }
                        if (typeof window.gbHideBootSplash === "function") {
                            window.gbHideBootSplash();
                        }
                    } catch (err) {
                        console.error("Failed to auto-open suite app:", err);
                    }
                }
            });

            // Listen to HTMX afterRequest event
            document.body.addEventListener("htmx:afterRequest", function (evt) {
                // Ensure Theme dropdown is re-injected if wiped
                if (window.ThemeManager) {
                    const container = document.getElementById(
                        "themeSelectorContainer",
                    );
                    if (container && !container.hasChildNodes()) {
                        window.ThemeManager.init();
                    }
                }
                // Re-evaluate auth state after every HTMX navigation
                if (typeof updateLoginButton === 'function') {
                    updateLoginButton();
                }
            });

            // Auth UI (sign-in / user menu) lives in js/sidebar.js — it renders
            // into the sidebar footer instead of the taskbar tray.

            // Desktop wallpaper: restore the user's saved choice and provide a
            // right-click popup with a Settings command (opens in a background tab).
            (function () {
                var savedWall = localStorage.getItem("gb-wallpaper") || "default";
                document.body.setAttribute("data-wallpaper", savedWall);

                var workspace = document.getElementById("desktop-content");
                if (!workspace) return;

                var menu = null;
                function closeDesktopMenu() {
                    if (menu) {
                        menu.remove();
                        menu = null;
                    }
                    document.removeEventListener("click", closeDesktopMenu);
                }

                function openDesktopMenu(x, y) {
                    closeDesktopMenu();
                    menu = document.createElement("div");
                    menu.className = "desktop-context-menu";
                    menu.style.left = Math.min(x, window.innerWidth - 190) + "px";
                    menu.style.top = Math.min(y, window.innerHeight - 150) + "px";
                    menu.innerHTML =
                        '<div class="desktop-context-item" onclick="if(window.openDeepLink){window.openDeepLink(\'settings\',{});}else{var t=window.open(\'/suite/settings/index.html\',\'_blank\');if(t)setTimeout(function(){window.focus();},50);}">' +
                        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>' +
                        '<span>Settings</span></div>' +
                        '<div class="desktop-context-item" onclick="window.location.reload()">' +
                        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10"/><path d="M20.49 15a9 9 0 0 1-14.85 3.36L1 14"/></svg>' +
                        '<span>Refresh</span></div>' +
                        '<div class="desktop-context-item" onclick="window.WindowManager && window.WindowManager.toggleStartMenu()">' +
                        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>' +
                        '<span>Start Menu</span></div>' +
                        '<div class="desktop-context-separator"></div>' +
                        '<div class="desktop-context-item" onclick="window.GBWidgets && window.GBWidgets.openPicker(event)">' +
                        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>' +
                        '<span>Add widget…</span></div>' +
                        '<div class="desktop-context-item" onclick="window.GBWidgets && window.GBWidgets.addWebPrompt()">' +
                        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>' +
                        '<span>Add web page widget…</span></div>';
                    document.body.appendChild(menu);
                    document.addEventListener("click", closeDesktopMenu);
                }

                workspace.addEventListener("contextmenu", function (e) {
                    // The desktop (wallpaper) menu is only for the wallpaper
                    // background itself, not for windows, icons or widgets.
                    if (e.target !== workspace) return;
                    e.preventDefault();
                    openDesktopMenu(e.clientX, e.clientY);
                });

                window.addEventListener("storage", function (e) {
                    if (e.key === "gb-wallpaper" && e.newValue) {
                        document.body.setAttribute("data-wallpaper", e.newValue);
                    }
                });

                // React to same-tab theme/wallpaper changes from the Settings
                // window (the `storage` event only fires across tabs).
                window.addEventListener("gb-theme-changed", function (e) {
                    if (e.detail && e.detail.theme) {
                        document.documentElement.setAttribute(
                          "data-theme",
                          e.detail.theme,
                        );
                    }
                });
                window.addEventListener("gb-wallpaper-changed", function (e) {
                    if (e.detail && e.detail.name) {
                        document.body.setAttribute(
                          "data-wallpaper",
                          e.detail.name,
                        );
                    }
                });
            })();

            // Widget pane + pinned launcher (#1160)
            (function () {
                if (window.GBWidgets) {
                    window.GBWidgets.init(document.getElementById("desktop-content"));
                }
                if (window.GBPinnedLauncher) {
                    window.GBPinnedLauncher.init();
                }
                // Per-user desktop shortcuts: drop Drive files here (#1188)
                if (window.GBDesktopShortcuts) {
                    window.GBDesktopShortcuts.init(document.getElementById("desktop-content"));
                }
                // Multitasking (#1155): virtual desktops, chrome (#1158), concierge (#1157)
                if (window.VirtualDesktops) {
                    window.VirtualDesktops.init(document.getElementById("desktop-content"));
                }
                if (window.ControlCenter) {
                    window.ControlCenter.init();
                }
                if (window.LockScreen) {
                    window.LockScreen.init();
                }
                if (window.GBToasts) {
                    window.GBToasts.init();
                }
                if (window.Spotlight) {
                    window.Spotlight.init();
                }
                if (window.AgentExecutor) {
                    window.AgentExecutor.init();
                }
            })();

            // Listen for auth state changes
            function initAuthUI() {
                if (typeof updateLoginButton === "function") updateLoginButton();
                window.addEventListener("gb:auth:login", function () {
                    if (typeof updateLoginButton === "function") updateLoginButton();
                });
                window.addEventListener("gb:auth:logout", function () {
                    if (typeof updateLoginButton === "function") updateLoginButton();
                });
                window.addEventListener("gb:auth:expired", function () {
                    if (typeof updateLoginButton === "function") updateLoginButton();
                });
            }

            if (window.GBSecurity && window.GBSecurity.initialized) {
                initAuthUI();
            } else {
                window.addEventListener("gb:security:ready", initAuthUI);
                setTimeout(initAuthUI, 1000);
            }

            // Taskbar clock. The date/time format comes from Settings ->
            // Language through the shared GBDateTime helper, and a preference
            // change re-renders immediately (#1339).
            (() => {
                function renderClock() {
                    const now = new Date();
                    const timeEl = document.getElementById("clock-time");
                    const dateEl = document.getElementById("clock-date");
                    const formatted = window.GBDateTime
                        ? window.GBDateTime.now()
                        : {
                              time: now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
                              date: now.toLocaleDateString(),
                          };
                    if (timeEl) timeEl.textContent = formatted.time;
                    if (dateEl) dateEl.textContent = formatted.date;
                }
                renderClock();
                setInterval(renderClock, 1000);
                window.addEventListener("storage", (e) => {
                    if (e.key === "gb-date-format" || e.key === "gb-time-format") renderClock();
                });
            })();
        
