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
(function () {
    "use strict";

    var S = window.VibeShell;

    function wm() {
        return typeof window.WindowManager !== "undefined" ? window.WindowManager : null;
    }

    function el(tag, cls, text) {
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

    /* #1271 — Chat button opens the shared Chat window as a NEW
       conversation (no session param = fresh) and pre-fills the input with a
       directive so the message is routed to the app currently running in
       vibe. The pre-fill is the RUNNING APP'S NAME (@qa-flow), never a long
       botbook file path — the user must see the app name in the input, not
       a path to a .md file. Falls back to the botbook directive only when
       no project is selected. */
    var CHAT_BOOK_DIRECTIVE =
        "@botbook/src/10-configuration-deployment/config-csv.md";
    function openChat() {
        var pid = S.projectId();
        var name = S.projectName();
        var kind = String(
            (typeof window.currentProjectKind !== "undefined" && window.currentProjectKind) ||
            (function () { try { return localStorage.getItem("gb-vibe-project-kind") || ""; } catch (e) { return ""; } })()
        ).toLowerCase();
        // #1386 follow-up — bot-kind Vibe projects ARE bots (bots row with
        // origin='vibe'): their Chat window binds to the bot identity via
        // the { bot: slug } deep-link so the WS session runs as that bot.
        // They are test bots, launcher-invisible — the Vibe window is their
        // only entry point. website/apps projects keep the @project mention
        // (they are agent contexts, not bots).
        if (pid && name && kind === "bot") {
            var slug = String(name).toLowerCase().replace(/\s+/g, "-");
            openSharedApp("chat", { bot: slug });
            return;
        }
        var message =
            pid && name && String(name) !== "vibe"
                ? "@" + name
                : CHAT_BOOK_DIRECTIVE;
        openSharedApp("chat", { message: message });
    }

    /* Commit is a popup dialog (the Source Control dialog), never toolbar
       commands — the toolbar carries just selectors + window buttons. */
    function openCommit() {
        if (window.VibeDialogs) {
            window.VibeDialogs.open("git", "Source Control");
            return;
        }
        openSharedApp("editor", {});
    }

    /* Project actions for the selected project: the classic sidebar (which
       carried ⓘ / 🗑 per project) is hidden in toolbar mode, so the toolbar
       must expose them or users cannot see the buttons at all. */
    function openProjectInfo() {
        var pid = S.projectId();
        if (!pid) {
            // Properties needs a project; never jump the user into the New
            // Project dialog here (it confused "Properties is broken" reports
            // when the selection was empty/stale past a reload).
            flashHint("SELECT A PROJECT FIRST");
            var sel = projectSelect();
            if (sel) sel.focus();
            return;
        }
        if (window.VibeWindows && typeof window.VibeWindows.openProjectInfo === "function") {
            window.VibeWindows.openProjectInfo();
            return;
        }
        if (typeof window.showProjectInfo === "function") {
            var match = knownProjects.find(function (p) {
                var id = p.project_id || p.id;
                return id != null && String(id) === String(pid);
            });
            window.showProjectInfo(match || {
                id: pid,
                name: pid,
                project_type: window.currentProjectKind || "",
            });
        }
    }

    function deleteSelectedProject() {
        var pid = S.projectId();
        if (!pid) {
            flashHint("SELECT A PROJECT FIRST");
            var sel = projectSelect();
            if (sel) sel.focus();
            return;
        }
        var match = knownProjects.find(function (p) {
            var id = p.project_id || p.id;
            return id != null && String(id) === String(pid);
        });
        if (typeof window.deleteProject === "function") {
            window.deleteProject(match || { project_id: pid, name: pid });
        } else if (typeof window.VibeAgents !== "undefined" && typeof window.VibeAgents.deleteProject === "function") {
            window.VibeAgents.deleteProject(match || { project_id: pid, name: pid });
        }
    }

    /* ── Inline SVG icons (custom, stroke style, no emoji) ─────────── */
    var ICONS = {
        Terminal: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>',
        Browser: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/></svg>',
        Chat: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>',
        "Runner Log": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>',
        "Knowledge Graph": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>',
        Canvas: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="M21 15l-5-5L5 21"/></svg>',
        Metrics: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>',
        "Source Control": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="6" cy="5" r="3"/><circle cx="6" cy="19" r="3"/><circle cx="18" cy="12" r="3"/><line x1="6" y1="8" x2="6" y2="16"/><line x1="8.59" y1="5.86" x2="15.42" y2="11.14"/><line x1="15.42" y1="12.86" x2="8.59" y2="18.14"/></svg>',
        Run: '<svg viewBox="0 0 24 24" fill="currentColor"><polygon points="6 3 20 12 6 21"/></svg>',
        // Rocket = publish to production (Run tests dev, Deploy ships prod).
        Deploy: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/><path d="M9 12H4s.55-3.03 2-4c1.62-1.08 5 0 5 0"/><path d="M12 15v5s3.03-.55 4-2c1.08-1.62 0-5 0-5"/></svg>',
        Pause: '<svg viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="4" width="5" height="16"/><rect x="14" y="4" width="5" height="16"/></svg>',
        Stop: '<svg viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="5" width="14" height="14"/></svg>',
        "New Project": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>',
        Properties: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
        Delete: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>',
        Editor: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>',
        Members: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>',
        Database: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>',
    };

    /* VB4-style big toolbar button: SVG icon over a short label. */
    function buildButton(label, iconKey, handler, extraCls) {
        var btn = el("button", "vibe-shell-tb-btn" + (extraCls ? " " + extraCls : ""));
        btn.type = "button";
        btn.title = label;
        btn.innerHTML =
            '<span class="vibe-shell-tb-icon">' + (ICONS[iconKey] || iconKey) + "</span>" +
            '<span class="vibe-shell-tb-label">' + label + "</span>";
        btn.addEventListener("click", handler);
        return btn;
    }

    /* ── Transient hint pill (no status bar — VB feedback = lights/hints) ── */
    var hintTimer = null;

    function flashHint(text) {
        var bar = document.getElementById("vibeShellToolbar");
        if (!bar) return;
        var pill = document.getElementById("vibeShellHint");
        if (!pill) {
            pill = el("div", "vibe-shell-hint");
            pill.id = "vibeShellHint";
            bar.appendChild(pill);
        }
        pill.textContent = text;
        pill.classList.add("show");
        if (hintTimer) clearTimeout(hintTimer);
        hintTimer = setTimeout(function () {
            pill.classList.remove("show");
        }, 3600);
    }

    /* ── Frozen UI during destructive operations (project delete) ──
       Sets a full-viewport overlay that blocks all pointer input and
       disables every toolbar button until the operation finishes and the
       project combo is refreshed (see deleteProject → setBusy(false)). */
    function setBusy(busy, label) {
        var existing = document.getElementById("vibeBusyOverlay");
        if (busy) {
            if (!existing) {
                var ov = el("div", "vibe-busy-overlay");
                ov.id = "vibeBusyOverlay";
                ov.style.cssText =
                    "position:fixed;inset:0;z-index:2147483000;display:flex;align-items:center;justify-content:center;" +
                    "background:rgba(8,10,20,.55);backdrop-filter:blur(2px);cursor:progress;";
                ov.innerHTML =
                    '<div style="background:#141628;border:1px solid rgba(255,255,255,.18);border-radius:12px;' +
                    'padding:22px 34px;text-align:center;color:#eee;font-size:13px;' +
                    'box-shadow:0 14px 40px rgba(0,0,0,.5)">' +
                    '<div style="width:26px;height:26px;margin:0 auto 12px;border:3px solid rgba(255,255,255,.2);' +
                    'border-top-color:#60a5fa;border-radius:50%;animation:vibe-busy-spin .8s linear infinite"></div>' +
                    "<b>" + (label || "Working…") + "</b></div>";
                if (!document.getElementById("vibeBusySpinStyle")) {
                    var style = el("style", null);
                    style.id = "vibeBusySpinStyle";
                    style.textContent = "@keyframes vibe-busy-spin{to{transform:rotate(360deg)}}";
                    document.head.appendChild(style);
                }
                document.body.appendChild(ov);
            } else {
                existing.style.display = "flex";
            }
        } else if (existing) {
            existing.remove();
        }
        document.body.classList.toggle("vibe-shell-busy", busy);
        var bar = document.getElementById("vibeShellToolbar");
        if (bar) {
            bar.classList.toggle("vibe-shell-busy", busy);
            if (typeof bar.__refreshCommandState === "function") bar.__refreshCommandState();
        }
    }

    /* ── Active-project dropdown ──────────────────────────────────── */
    var knownProjects = [];

    function projectSelect() {
        return document.getElementById("vibeShellProjectSelect");
    }

    function syncProjectSelect() {
        var sel = projectSelect();
        if (!sel) return;
        var id = S.projectId();
        if (!id) return;
        Array.prototype.forEach.call(sel.options, function (opt) {
            opt.selected = String(opt.value) === String(id);
        });
    }

    function loadProjects() {
        var sel = projectSelect();
        if (!sel || typeof vibeApi !== "function") return Promise.resolve();
        return vibeApi("/api/vibe/projects")
            .then(function (data) {
                var projects =
                    (data && data.success && data.projects) ||
                    (data && data.projects) ||
                    [];
                knownProjects = projects;
                sel.innerHTML = "";
                if (!projects.length) {
                    sel.disabled = true;
                    sel.appendChild(el("option", null, "No projects"));
                    // Only wipe a persisted selection when the list is
                    // GENUINELY empty. An anonymous/401 first paint returns
                    // an error envelope here, and clearing would null a valid
                    // stored project (which then made Properties/selected
                    // buttons fall back to 'SELECT A PROJECT FIRST').
                    var listReallyEmpty = data && !data.error && data.success !== false;
                    if (listReallyEmpty) clearStaleSelection();
                    loadBranches();
                    return;
                }
                sel.disabled = false;
                projects.forEach(function (p) {
                    var id = p.project_id || p.id;
                    var name = p.name || p.project_type || "Unnamed project";
                    var opt = el("option", null, name);
                    opt.value = id == null ? "" : String(id);
                    sel.appendChild(opt);
                });
                if (!S.projectId()) {
                    var storedId = "";
                    // Prefer the durable (localStorage) selection so the last
                    // used project — not projects[0] — is restored across
                    // sessions; fall back to the in-session value.
                    try { storedId = localStorage.getItem("gb-vibe-project-id") || 
                                     sessionStorage.getItem("gb-vibe-project-id") || ""; } catch (ignore) { }
                    var preferred = projects.find(function (p) {
                        var id = p.project_id || p.id;
                        return storedId && String(id) === String(storedId);
                    }) || projects.find(function (p) {
                        return String(p.status || "").toLowerCase() === "active";
                    }) || projects[0];
                    if (preferred && typeof applyProjectSelection === "function") {
                        applyProjectSelection(preferred);
                    }
                } else {
                    // #1402 — the persisted selection can outlive its project
                    // (deleted in another session, or evicted by the branch
                    // cap). Project-gated commands (Run, Deploy, ...) must
                    // stay disabled when the id no longer resolves, instead of
                    // firing "SELECT A PROJECT FIRST" onto a ghost project.
                    var stillExists = projects.some(function (p) {
                        var id = p.project_id || p.id;
                        return id != null && String(id) === String(S.projectId());
                    });
                    if (!stillExists) clearStaleSelection();
                }
                syncProjectSelect();
                loadBranches();
            })
            .catch(function () { /* dropdown stays with its previous content */ });
    }

    /* Drop a persisted selection that no longer resolves to a real project so
       the toolbar re-evaluates to the disabled state (New Project stays
       enabled). Shared by the empty-list and the stale-id paths. */
    function clearStaleSelection() {
        try {
            sessionStorage.removeItem("gb-vibe-project-id");
            localStorage.removeItem("gb-vibe-project-id");
            sessionStorage.removeItem("gb-vibe-project-name");
            localStorage.removeItem("gb-vibe-project-name");
        } catch (e) { /* storage unavailable */ }
        if (typeof window.currentProjectId !== "undefined") window.currentProjectId = null;
        if (typeof window.currentProject !== "undefined") window.currentProject = "";
        var bar = document.getElementById("vibeShellToolbar");
        if (bar && typeof bar.__refreshCommandState === "function") bar.__refreshCommandState();
    }

    /* Dev-VM lifecycle (#1271): the dev VM is always on since project
       creation; it stops when the vibe window closes or the user switches to
       another project (the next Run restarts it). Production VMs are only
       activated by Deploy and stay on forever. */
    function stopDevVm(projectId) {
        if (!projectId) return Promise.resolve();
        return vibeApi("/api/vibe/projects/" + encodeURIComponent(projectId) + "/vms")
            .then(function (data) {
                var vms = (data && data.vms) || [];
                var dev = vms.find(function (v) {
                    return String(v.env || "") === "development";
                }) || vms[0];
                if (!dev || !dev.id) return null;
                return vibeApi("/api/vibe/vms/" + encodeURIComponent(dev.id) + "/stop", { method: "POST" });
            })
            .catch(function () { return null; });
    }

    function onProjectChange() {
        var sel = projectSelect();
        if (!sel) return;
        // Stop the PREVIOUS project's dev VM (the lifecycle is automatic:
        // one project at a time in dev). The new selection's VM starts on
        // Run / is already running if it was the same project.
        var previousId = S.projectId();
        var id = sel.value;
        if (previousId && String(previousId) !== String(id)) {
            stopDevVm(previousId);
        }
        var match = knownProjects.find(function (p) {
            var pid = p.project_id || p.id;
            return pid != null && String(pid) === String(id);
        });
        if (match && typeof applyProjectSelection === "function") {
            applyProjectSelection(match);
        }
        // Switching project invalidates every floating accessory (canvas
        // project.draw, graph runs, metrics, run dock, dialogs): close them
        // so stale project-scoped content cannot linger on screen.
        if (window.VibeWindows && typeof window.VibeWindows.closeVibeSubwindows === "function") {
            window.VibeWindows.closeVibeSubwindows();
        }
        loadBranches();
    }

    /* ── Branch dropdown (toolbar) ────────────────────────────────── */
    function branchSelect() {
        return document.getElementById("vibeShellBranchSelect");
    }

    /* Branch combo over the REAL project workspace repo (#1271). The old
       /api/git/* endpoints resolve any non-/tmp repo to a fixed stub, so the
       combo never showed the project's branches (and never listed the
       release/prev-* rollback branches Deploy creates). */
    function loadBranches() {
        var sel = branchSelect();
        if (!sel) return;
        var pid = S.projectId();
        if (!pid) {
            sel.disabled = true;
            sel.innerHTML = "";
            sel.appendChild(el("option", null, "—"));
            return;
        }
        vibeApi("/api/vibe/projects/" + encodeURIComponent(pid) + "/branches")
            .then(function (data) {
                var branches = (data && data.branches) || [];
                sel.innerHTML = "";
                if (!Array.isArray(branches) || !branches.length) {
                    sel.disabled = true;
                    sel.appendChild(el("option", null, "main"));
                    return;
                }
                sel.disabled = false;
                branches.forEach(function (b) {
                    var name = typeof b === "string" ? b : (b.name || "");
                    if (!name) return;
                    var opt = el("option", null, name);
                    opt.value = name;
                    if (typeof b === "object" && b.current) opt.selected = true;
                    sel.appendChild(opt);
                });
            })
            .catch(function () {
                sel.disabled = true;
                sel.innerHTML = "";
                sel.appendChild(el("option", null, "—"));
            });
    }

    function onBranchChange() {
        var sel = branchSelect();
        if (!sel || !sel.value) return;
        var pid = S.projectId();
        if (!pid) return;
        vibeApi("/api/vibe/projects/" + encodeURIComponent(pid) + "/branches/" +
            encodeURIComponent(sel.value), { method: "POST" })
            .then(function () { loadBranches(); })
            .catch(function () { });
    }

    /* ── Transport: Run starts the selected project through the authoritative
           Vibe runner, then opens its preview; Stop cancels the run and closes
           the preview. ── */
    function setRunVisual(running) {
        var bar = document.getElementById("vibeShellToolbar");
        if (bar) bar.classList.toggle("vibe-shell-running", running);
    }

    function previewOpen() {
        var mgr = wm();
        return !!(mgr && mgr.getWindow && mgr.getWindow("browser"));
    }

    /* #1271 — Run starts the app as a REAL process in the dev VM: the run
       endpoint pushes the workspace files into the dev container, starts node
       (or a static server) as a systemd service and returns the exposed URL.
       The app is then visible in the project terminal's `ps`. Falls back to
       the static workspace stream when the VM is unavailable. */
    function startDevVm(projectId) {
        return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId) + "/run", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({}),
        })
            .then(function (r) { return r.json(); })
            .then(function (data) {
                if (data && data.success && data.url) {
                    return { url: data.url, vm: true, container: data.container };
                }
                throw new Error((data && data.error) || "dev VM run failed");
            });
    }

    function openPreview() {
        var projectId = S.projectId();
        if (!projectId) {
            flashHint("SELECT A PROJECT FIRST");
            var sel = projectSelect();
            if (sel) sel.focus();
            return;
        }
        var selectedProject = knownProjects.find(function (project) {
            var id = project.project_id || project.id;
            return id != null && String(id) === String(projectId);
        });
        var projectName = (selectedProject && selectedProject.name) || S.projectName();
        // 1) Try to start the app on the dev VM (real node process).
        // 2) Fall back to the static workspace stream when the VM is absent.
        // Run is a deterministic "start the app on the dev VM" action: it
        // pushes the workspace into the container, restarts the node service,
        // and reveals the Browser. It must NOT invoke the agent loop
        // (VibeTransport.play), which spends LLM tool calls running/verifying
        // the project — that belongs to the Chat agent, not the Run button.
        startDevVm(projectId)
            .then(function (vm) {
                openBrowser(vm.url);
                setRunVisual(true);
                flashHint("RUNNING " + String(projectName).toUpperCase() + " ON THE DEV VM");
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
                        flashHint("RUNNING " + String(projectName).toUpperCase() + " (STATIC PREVIEW — NO VM)");
                    })
                    .catch(function (err) {
                        setRunVisual(false);
                        openBrowser(null);
                        flashHint((err && err.message ? err.message : "No preview available") + " — deploy the project to see your app");
                    });
            });
    }

    function pausePreview() {
        // Pause belongs to the active Vibe run. It must not hide the Browser
        // preview, because that makes the transport controls ambiguous.
        var transport = window.VibeTransport;
        if (transport && typeof transport.pause === "function") {
            transport.pause();
            return;
        }
        flashHint("NO RUN TO PAUSE");
    }

    function closePreview() {
        var transport = window.VibeTransport;
        if (transport && typeof transport.hasActiveRun === "function" && transport.hasActiveRun()) {
            transport.stop();
        }
        var mgr = wm();
        if (mgr && mgr.getWindow && mgr.getWindow("browser")) {
            mgr.close("browser");
        }
        setRunVisual(false);
        flashHint("STOPPED");
    }

    // Deploy = publish to production. Run tests in dev; Deploy is the only
    // path to production (approval-gated deploy pipeline, pipeline_mode
    // "deploy"). Distinct styling so the dev/prod split reads at a glance.
    function deployProject() {
        var projectId = S.projectId();
        if (!projectId) {
            // Reload/restore can leave the in-memory selection empty while the
            // last-used project is still persisted (#deploy-the-selected).
            // Recover it before bailing out so Deploy never mints a junk
            // project from the intent slug.
            try {
                var savedId = localStorage.getItem("gb-vibe-project-id");
                var savedName = localStorage.getItem("gb-vibe-project-name");
                if (savedId && typeof applyProjectSelection === "function") {
                    var match = knownProjects.find(function (p) {
                        var pid = p.project_id || p.id;
                        return pid != null && String(pid) === String(savedId);
                    });
                    applyProjectSelection(match || { project_id: savedId, name: savedName || "" }, false);
                    projectId = savedId;
                }
            } catch (_e) { /* storage unavailable */ }
        }
        if (!projectId) {
            flashHint("SELECT A PROJECT FIRST");
            var sel = projectSelect();
            if (sel) sel.focus();
            return;
        }
        var selectedProject = knownProjects.find(function (project) {
            var id = project.project_id || project.id;
            return id != null && String(id) === String(projectId);
        });
        var projectName = (selectedProject && selectedProject.name) || S.projectName();
        setRunVisual(true);
        flashHint("DEPLOYING " + String(projectName).toUpperCase() + " TO PRODUCTION");
        // The deploy pipeline snapshots the current deployment into a
        // release/prev-<ts> branch; reload the branch combo when the run
        // finishes so the rollback branch is immediately switchable.
        var poll = setInterval(function () {
            var t = window.VibeTransport;
            var done = !t || typeof t.hasActiveRun !== "function" || !t.hasActiveRun();
            if (done) {
                clearInterval(poll);
                loadBranches();
            }
        }, 2500);
        var transport = window.VibeTransport;
        if (transport && typeof transport.deploy === "function") {
            transport.deploy();
            return;
        }
        if (window.VibeRun && typeof window.VibeRun.deploy === "function") {
            window.VibeRun.deploy();
        }
    }

    function openRunnerLog() {
        // The runner log window IS the Run Dock; it must be tall (80% of the
        // desktop) and docked at the top so the board/log/sessions all show.
        if (window.VibeWindows && window.VibeWindows.openRunDock) window.VibeWindows.openRunDock();
    }

    /* Toolbar — a 2-row grid, never more than 2 rows (product spec #1383):

       │ RUN  DEPLOY │ Project │ [New Project][Browser][Run Logger] │ ✕ │
       │ (2 rows)    │ Branch  │ [Chat] [Terminal] [Properties]     │
       │             │         │ … three buttons per line           │

       Run and Deploy sit together at the LEFT (double-height primaries);
       the Project/Branch combos follow; then groups of stacked window
       buttons, three per line; ✕ closes every vibe window at once. */
    function build() {
        if (document.getElementById("vibeShellToolbar")) return;
        var container = document.getElementById("vibeWindow");
        if (!container) return;

        var bar = el("div", "vibe-shell-toolbar vibe-shell-tb-grid");
        bar.id = "vibeShellToolbar";
        bar.setAttribute("role", "toolbar");
        bar.setAttribute("aria-label", "Vibe commands");

        /* ── RUN + DEPLOY — the two primaries, together at LEFT ─── */
        var runGroup = el("div", "vibe-shell-tb-group vibe-shell-tb-run-group");
        runGroup.appendChild(buildButton("Run", "Run", openPreview, "vibe-shell-tb-run"));
        bar.appendChild(runGroup);

        var deployGroup = el("div", "vibe-shell-tb-group vibe-shell-tb-deploy-group");
        deployGroup.appendChild(buildButton("Deploy", "Deploy", deployProject, "vibe-shell-tb-deploy"));
        bar.appendChild(deployGroup);

        /* ── Combos — two rows: Project on top, Branch below ────── */
        var selectors = el("div", "vibe-shell-tb-group vibe-shell-tb-selectors");

        var projWrap = el("label", "vibe-shell-tb-field");
        projWrap.appendChild(el("span", "vibe-shell-tb-field-label", "Project"));
        var sel = el("select", "vibe-shell-project-select");
        sel.id = "vibeShellProjectSelect";
        sel.title = "Active project";
        sel.setAttribute("aria-label", "Active project");
        sel.disabled = true;
        sel.appendChild(el("option", null, "Loading…"));
        sel.addEventListener("change", onProjectChange);
        projWrap.appendChild(sel);
        selectors.appendChild(projWrap);

        var brWrap = el("label", "vibe-shell-tb-field");
        brWrap.appendChild(el("span", "vibe-shell-tb-field-label", "Branch"));
        var brSel = document.createElement("select");
        brSel.id = "vibeShellBranchSelect";
        brSel.className = "vibe-shell-project-select vibe-shell-branch-select";
        brSel.title = "Active branch";
        brSel.setAttribute("aria-label", "Active branch");
        brSel.disabled = true;
        brSel.appendChild(el("option", null, "—"));
        brSel.addEventListener("change", onBranchChange);
        brWrap.appendChild(brSel);
        selectors.appendChild(brWrap);
        bar.appendChild(selectors);

    /* ── Window commands — 3 rows × 4 columns (#1383 review): a single
       full-height separator spans the three lines between column groups ── */
        var cmds = el("div", "vibe-shell-tb-group vibe-shell-tb-cmds");
        // 12 commands in a 3×4 grid — exactly the revised issue sketch:
        //   row 1: New Project | Browser        | Editor         | Canvas
        //   row 2: Chat        | Terminal       | Database       | Knowledge Graph
        //   row 3: Properties  | Run Logger     | Source Control | Members
        var editorPid = S.projectId();
        var defs = [
            ["New Project", "New Project", function () { if (window.VibeNewProject) window.VibeNewProject.open(); else if (window.VibeWindows) window.VibeWindows.openNewProject(); }, "vibe-shell-tb-new"],
            ["Browser", "Browser", function () {
                var pid = S.projectId();
                if (!pid) { openBrowser(null); return; }
                startDevVm(pid)
                    .then(function (vm) { openBrowser(vm.url); })
                    .catch(function () {
                        workspaceServeUrl(pid)
                            .then(function (url) {
                                if (url) return url;
                                return resolvePreviewUrl(pid);
                            })
                            .then(function (url) { openBrowser(url); })
                            .catch(function () { openBrowser(null); });
                    });
            }, null],
            ["Editor", "Editor", function () {
                var pid = S.projectId() || editorPid;
                openSharedApp("editor", pid ? { project: pid } : {});
            }, null],
            ["Canvas", "Canvas", function () { if (window.VibeWindows) window.VibeWindows.openCanvas(); }, null],
            ["Chat", "Chat", openChat, null],
            ["Terminal", "Terminal", openTerminal, "vibe-shell-tb-terminal"],
            // Database opens the SQL schema editor (same dialog the ribbon's
            // Database command used — #1189).
            ["Database", "Database", function () { if (window.VibeDialogs) window.VibeDialogs.open("db", "Database Schema"); }, null],
            ["Knowledge Graph", "Knowledge Graph", function () { if (window.VibeWindows) window.VibeWindows.openGraph(); }, null],
            ["Properties", "Properties", openProjectInfo, "vibe-shell-tb-info"],
            ["Run Logger", "Runner Log", openRunnerLog, null],
            ["Source Control", "Source Control", openCommit, null],
            // Members was only reachable from the hidden legacy ribbon — the
            // toolbar button keeps it reachable (no ghost windows).
            ["Members", "Members", function () { if (window.VibeWindows) window.VibeWindows.openMembers(); }, null],
        ];
        defs.forEach(function (d, i) {
            var btn = buildButton(d[0], d[1], d[2], d[3]);
            // Column c (0..3) → grid col 1,3,5,7; separator columns 2,4,6
            btn.style.gridColumn = String((i % 4) * 2 + 1);
            btn.style.gridRow = String(Math.floor(i / 4) + 1);
            cmds.appendChild(btn);
        });
        // One separator per column boundary, spanning all 3 rows (Md-file
        // style single rule instead of a mark on every line).
        for (var s = 0; s < 3; s++) {
            var sep = el("span", "vibe-shell-tb-sep vibe-shell-tb-sep3");
            sep.setAttribute("aria-hidden", "true");
            sep.style.gridColumn = String(s * 2 + 2);
            sep.style.gridRow = "1 / 4";
            cmds.appendChild(sep);
        }
        bar.appendChild(cmds);

        // #1325 — project-gated commands: when no project is selected every
        // command that needs one is disabled; New Project and Close All stay
        // enabled. Re-evaluated whenever the selection changes.
        function refreshCommandState() {
            var hasProject = !!S.projectId();
            var kind = selectedProjectKind();
            var frozen = document.body.classList.contains("vibe-shell-busy");
            Array.prototype.forEach.call(bar.querySelectorAll(".vibe-shell-tb-btn"), function (b) {
                if (b.classList.contains("vibe-shell-tb-new") || b.classList.contains("vibe-shell-tb-closeall")) {
                    // New Project and Close-all stay disabled while frozen too.
                    b.disabled = frozen;
                    if (frozen) b.classList.add("vibe-shell-tb-disabled");
                    else b.classList.remove("vibe-shell-tb-disabled");
                    return;
                }
                b.disabled = frozen || !hasProject;
                b.classList.toggle("vibe-shell-tb-disabled", frozen || !hasProject);
            });
            if (frozen) return;
            // #1403 — website and bot projects have no dev VM/container, so a
            // terminal there would only ever open a dead pane. Keep the button
            // visible but inert (New Project and Close All remain enabled).
            var terminal = bar.querySelector(".vibe-shell-tb-terminal");
            if (terminal) {
                var noTerminal = !hasProject || kind === "website" || kind === "bot";
                terminal.disabled = noTerminal;
                terminal.classList.toggle("vibe-shell-tb-disabled", noTerminal);
                terminal.title = noTerminal ? "No terminal for this project kind" : "Terminal";
            }
        }

        /* Resolve the selected project's kind ('bot' | 'website' | 'apps')
           from the live selection first, falling back to the loaded list. */
        function selectedProjectKind() {
            if (typeof window.currentProjectKind !== "undefined" && window.currentProjectKind) {
                return String(window.currentProjectKind).toLowerCase();
            }
            var pid = S.projectId();
            if (pid) {
                var p = knownProjects.find(function (x) {
                    var id = x.project_id || x.id;
                    return id != null && String(id) === String(pid);
                });
                if (p && p.project_type) return String(p.project_type).toLowerCase();
            }
            return "";
        }
        bar.__selectedProjectKind = selectedProjectKind;
        bar.__refreshCommandState = refreshCommandState;
        refreshCommandState();
        document.addEventListener("gb:vibe-project", refreshCommandState);
        document.addEventListener("gb:vibe-project-created", refreshCommandState);


        /* ── Close-all ✕ — closes every vibe window at once ─────── */
        var closeGroup = el("div", "vibe-shell-tb-group vibe-shell-tb-close-group");
        var closeBtn = el("button", "vibe-shell-tb-btn vibe-shell-tb-closeall");
        closeBtn.type = "button";
        closeBtn.title = "Close all vibe windows";
        closeBtn.innerHTML = '<span class="vibe-shell-tb-icon">✕</span>';
        closeBtn.addEventListener("click", function () {
            if (window.VibeWindows && typeof window.VibeWindows.closeVibeSubwindows === "function") {
                window.VibeWindows.closeVibeSubwindows();
            }
        });
        closeGroup.appendChild(closeBtn);
        bar.appendChild(closeGroup);

        // The rule in 90_shell.css used to hide the whole body; keep the old
        // ribbon tree out of the light entirely so there is no second command
        // bar and no dead chrome.
        var ribbon = container.querySelector(".vibe-ribbon");
        if (ribbon) ribbon.style.display = "none";

        var body = container.querySelector(".vibe-body");
        if (body && body.parentNode) {
            body.parentNode.insertBefore(bar, body);
        } else {
            container.insertBefore(bar, container.firstChild);
        }

        /* Run/idle status lives in the WINDOW STATUS BAR (product spec: no
           badge/status chip inside the command bar — the per-window inverted
           bevel status bar carries it). Mirror the authoritative legacy
           ribbon status text into #window-vibe .window-statusbar-status so
           there is exactly one status readout, in the window footer, not a
           pill inside the toolbar. */
        var statusBarTimer = setInterval(function () {
            var src = document.getElementById("vibeRibbonStatus");
            var statusEl = document.querySelector("#window-vibe .window-statusbar-status");
            if (!src || !statusEl) return;
            var text = String(src.textContent || "").trim();
            if (!text) return;
            statusEl.textContent = text;
            var state = /running|execut/i.test(text) ? "running"
                : /paus|approv/i.test(text) ? "paused"
                : /fail|error|stop/i.test(text) ? "failed"
                : "idle";
            statusEl.dataset.state = state;
            document.dispatchEvent(new CustomEvent("gb:vibe-status", { detail: { status: text, state: state } }));
        }, 800);
        if (bar && bar.__statusBarTimer && typeof bar.__statusBarTimer === "number") clearInterval(bar.__statusBarTimer);
        bar.__statusBarTimer = statusBarTimer;

        loadProjects();
        document.addEventListener("gb:vibe-project", function () {
            // #1307 — the delete path dispatches `gb:vibe-project` right after
            // loadVibeProjects(): a combo reload here removes the deleted
            // site from the toolbar dropdown instead of leaving it listed.
            loadProjects();
            syncProjectSelect();
            loadBranches();
        });
        // A project created through the New Project dialog must appear in the
        // toolbar combo immediately — the dialog dispatches
        // `gb:vibe-project-created` after the row is committed, so reload the
        // list (it also re-syncs the active selection) instead of leaving the
        // combo on its stale options.
        document.addEventListener("gb:vibe-project-created", function () {
            loadProjects();
        });
    }

    window.VibeShell.toolbar = {
        build: build,
        loadProjects: loadProjects,
        loadBranches: loadBranches,
        openTerminal: openTerminal,
        openBrowser: openBrowser,
        openProjectApp: openProjectApp,
        openChat: openChat,
        openCommit: openCommit,
        openPreview: openPreview,
        deployProject: deployProject,
        stopDevVm: stopDevVm,
        pausePreview: pausePreview,
        closePreview: closePreview,
        flashHint: flashHint,
        setBusy: setBusy,
        // Shared with palettes/dialogs so all Vibe chrome uses one SVG set.
        icons: ICONS,
    };
})();