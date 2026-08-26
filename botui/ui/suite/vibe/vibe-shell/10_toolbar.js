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
            window.open(app.hxGet, "_blank", "noopener");
        }
    }

    /* ── Shared app openers ────────────────────────────────────────── */
    function openTerminal() {
        var pid = S.projectId();
        openSharedApp("terminal", pid ? { project: pid } : {});
    }

    function builtInProjectUrl(project) {
        var name = String(project && (project.name || project.project_type) || "").toLowerCase();
        if (name.indexOf("calculator") !== -1) {
            return window.location.origin + "/suite/calculator/calculator.html?preview=1";
        }
        return "";
    }

    function resolvePreviewUrl(projectId) {
        if (!projectId) return Promise.reject(new Error("Select a project first"));
        return vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(projectId))
            .then(function (r) { return r.json(); })
            .then(function (projectData) {
                if (projectData && projectData.success === false) throw new Error(projectData.error || "Project lookup failed");
                var project = projectData && projectData.project;
                var builtInUrl = builtInProjectUrl(project);
                if (builtInUrl) return builtInUrl;
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
            openSharedApp("browser", { url: url });
            return;
        }
        openSharedApp("browser", {});
    }

    function openChat() {
        // The Vibe assistant IS the shared Chat window — same window, same
        // context; it also works standalone from the desktop shell.
        openSharedApp("chat", {});
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

    /* ── Inline SVG icons (custom, stroke style, no emoji) ─────────── */
    var ICONS = {
        Terminal: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>',
        Browser: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/></svg>',
        Chat: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>',
        "Runner Log": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg>',
        "Knowledge Graph": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/><line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/></svg>',
        Canvas: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="M21 15l-5-5L5 21"/></svg>',
        Metrics: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>',
        Run: '<svg viewBox="0 0 24 24" fill="currentColor"><polygon points="6 3 20 12 6 21"/></svg>',
        Hide: '<svg viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="4" width="5" height="16"/><rect x="14" y="4" width="5" height="16"/></svg>',
        Stop: '<svg viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="5" width="14" height="14"/></svg>',
        "New Project": '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>',
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
        if (!sel || typeof vibeApi !== "function") return;
        vibeApi("/api/vibe/projects")
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
                    try { storedId = sessionStorage.getItem("gb-vibe-project-id") || ""; } catch (ignore) { }
                    var preferred = projects.find(function (p) {
                        var id = p.project_id || p.id;
                        return storedId && String(id) === String(storedId);
                    }) || projects.find(function (p) {
                        return String(p.status || "").toLowerCase() === "active";
                    }) || projects[0];
                    if (preferred && typeof applyProjectSelection === "function") {
                        applyProjectSelection(preferred);
                    }
                }
                syncProjectSelect();
                loadBranches();
            })
            .catch(function () { /* dropdown stays with its previous content */ });
    }

    function onProjectChange() {
        var sel = projectSelect();
        if (!sel) return;
        var id = sel.value;
        var match = knownProjects.find(function (p) {
            var pid = p.project_id || p.id;
            return pid != null && String(pid) === String(id);
        });
        if (match && typeof applyProjectSelection === "function") {
            applyProjectSelection(match);
        }
        loadBranches();
    }

    /* ── Branch dropdown (toolbar) ────────────────────────────────── */
    function branchSelect() {
        return document.getElementById("vibeShellBranchSelect");
    }

    function loadBranches() {
        var sel = branchSelect();
        if (!sel) return;
        if (!S.projectId()) {
            sel.disabled = true;
            sel.innerHTML = "";
            sel.appendChild(el("option", null, "—"));
            return;
        }
        vibeApi("/api/git/branches?repo=" + encodeURIComponent(S.projectName()))
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
        vibeApi("/api/git/branch/" + encodeURIComponent(sel.value) +
            "?repo=" + encodeURIComponent(S.projectName()), { method: "POST" })
            .then(function () { loadBranches(); })
            .catch(function () { });
    }

    /* ── Transport (VB-style): Run opens the project in the shared Browser
           window, Hide minimizes it, Stop closes it. No status bar: the
           Run button lights up while the preview window is open. ── */
    function setRunVisual(running) {
        var bar = document.getElementById("vibeShellToolbar");
        if (bar) bar.classList.toggle("vibe-shell-running", running);
    }

    function previewOpen() {
        var mgr = wm();
        return !!(mgr && mgr.getWindow && mgr.getWindow("browser"));
    }

    function openPreview() {
        var projectId = S.projectId();
        if (!projectId) {
            flashHint("SELECT A PROJECT FIRST");
            var sel = projectSelect();
            if (sel) sel.focus();
            return;
        }
        var selectedProjectUrl = builtInProjectUrl({ name: S.projectName() });
        if (selectedProjectUrl) {
            openBrowser(selectedProjectUrl);
            setRunVisual(true);
            flashHint("RUNNING " + S.projectName().toUpperCase() + " IN THE BROWSER");
            return;
        }
        resolvePreviewUrl(projectId)
            .then(function (url) {
                openBrowser(url);
                setRunVisual(true);
                flashHint("RUNNING " + S.projectName().toUpperCase() + " IN THE BROWSER");
            })
            .catch(function (err) {
                // No live preview yet: still open the shared Browser window so
                // the user has a place to go, and say why the app is not loaded.
                setRunVisual(false);
                openBrowser(null);
                flashHint((err && err.message ? err.message : "No preview available") + " — deploy the project to see your app");
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

    function openRunnerLog() {
        if (window.VibeWindows && window.VibeWindows.openRunDock) window.VibeWindows.openRunDock();
    }

    function buildTransport() {
        var group = el("div", "vibe-shell-tb-group vibe-shell-tb-transport");
        group.appendChild(buildButton("Run", "Run", openPreview, "vibe-shell-tb-run"));
        group.appendChild(buildButton("Pause", "Hide", pausePreview, "vibe-shell-tb-pause"));
        group.appendChild(buildButton("Stop", "Stop", closePreview, "vibe-shell-tb-stop"));
        return group;
    }

    /* The single command row. Transport first so the primary controls stay
       visible; selectors and window buttons follow in the same row. */
    function build() {
        if (document.getElementById("vibeShellToolbar")) return;
        var container = document.getElementById("vibeWindow");
        if (!container) return;

        var bar = el("div", "vibe-shell-toolbar");
        bar.id = "vibeShellToolbar";
        bar.setAttribute("role", "toolbar");
        bar.setAttribute("aria-label", "Vibe commands");

        // Transport is deliberately first: Run/Pause/Stop are the primary
        // IDE controls and remain at the left edge on narrow windows.
        bar.appendChild(buildTransport());

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

        // Shared desktop apps (also available from the desktop shell launcher).
        var apps = el("div", "vibe-shell-tb-group vibe-shell-window-buttons");
        apps.appendChild(buildButton("Terminal", "Terminal", openTerminal));
        apps.appendChild(buildButton("Browser", "Browser", function () { openBrowser(null); }));
        apps.appendChild(buildButton("Chat", "Chat", openChat));
        bar.appendChild(apps);

        // Vibe tool windows with clear labels.
        var tools = el("div", "vibe-shell-tb-group vibe-shell-window-buttons");
        tools.appendChild(buildButton("Runner Log", "Runner Log", openRunnerLog));
        tools.appendChild(buildButton("Knowledge Graph", "Knowledge Graph", function () { if (window.VibeWindows) window.VibeWindows.openGraph(); }));
        tools.appendChild(buildButton("Canvas", "Canvas", function () { if (window.VibeWindows) window.VibeWindows.openCanvas(); }));
        tools.appendChild(buildButton("Metrics", "Metrics", function () { if (window.VibeWindows) window.VibeWindows.openMetrics(); }));
        bar.appendChild(tools);

        // No commit commands on the toolbar (VB design): committing is a
        // popup opened from the Source Control dialog only.
        var actions = el("div", "vibe-shell-tb-group vibe-shell-window-buttons");
        actions.appendChild(buildButton("New Project", "New Project", function () { if (window.VibeWindows) window.VibeWindows.openNewProject(); }));
        bar.appendChild(actions);

        var ribbon = container.querySelector(".vibe-ribbon");
        if (ribbon && ribbon.parentNode) {
            ribbon.parentNode.insertBefore(bar, ribbon);
        } else {
            container.insertBefore(bar, container.firstChild);
        }

        loadProjects();
        document.addEventListener("gb:vibe-project", function () {
            syncProjectSelect();
            loadBranches();
        });
    }

    window.VibeShell.toolbar = {
        build: build,
        loadProjects: loadProjects,
        loadBranches: loadBranches,
        openTerminal: openTerminal,
        openBrowser: openBrowser,
        openChat: openChat,
        openCommit: openCommit,
        openPreview: openPreview,
        pausePreview: pausePreview,
        closePreview: closePreview,
        flashHint: flashHint,
    };
})();