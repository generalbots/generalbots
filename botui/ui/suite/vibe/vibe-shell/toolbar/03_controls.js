/* Vibe Shell toolbar — split from vibe-shell/10_toolbar.js for #1731 */

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
        var btn = toolbarEl("button", "vibe-shell-tb-btn" + (extraCls ? " " + extraCls : ""));
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
            pill = toolbarEl("div", "vibe-shell-hint");
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
                var ov = toolbarEl("div", "vibe-busy-overlay");
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
                    var style = toolbarEl("style", null);
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
                    sel.appendChild(toolbarEl("option", null, "No projects"));
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
                    var opt = toolbarEl("option", null, name);
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
            sel.appendChild(toolbarEl("option", null, "—"));
            return;
        }
        vibeApi("/api/vibe/projects/" + encodeURIComponent(pid) + "/branches")
            .then(function (data) {
                var branches = (data && data.branches) || [];
                sel.innerHTML = "";
                if (!Array.isArray(branches) || !branches.length) {
                    sel.disabled = true;
                    sel.appendChild(toolbarEl("option", null, "main"));
                    return;
                }
                sel.disabled = false;
                branches.forEach(function (b) {
                    var name = typeof b === "string" ? b : (b.name || "");
                    if (!name) return;
                    var opt = toolbarEl("option", null, name);
                    opt.value = name;
                    if (typeof b === "object" && b.current) opt.selected = true;
                    sel.appendChild(opt);
                });
            })
            .catch(function () {
                sel.disabled = true;
                sel.innerHTML = "";
                sel.appendChild(toolbarEl("option", null, "—"));
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
