/* Vibe Shell toolbar — split from vibe-shell/10_toolbar.js for #1731 */

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

    /* #1409 follow-up — progress/state readouts. The Vibe window footer is
       the one status bar (product spec: the inverted bevel status bar carries
       run state) — write to the authoritative ribbon sink and mirror it into
       the window footer exactly like the 800ms poll does, but instantly. */
    function toolbarSetVibeStatus(text, state) {
        var el = document.getElementById("vibeRibbonStatus");
        if (el) {
            el.textContent = text || "";
            el.className = "vibe-ribbon-status" + (state === "running" ? " running" : state === "failed" ? " failed" : "");
            el.style.display = text ? "" : "none";
        }
        // #1849 — publish through the shared status contract. Keep the direct
        // fallback so an older desktop shell (without 07_status.js) still
        // shows the run state in the footer.
        if (window.GBStatus && typeof window.GBStatus.set === "function") {
            window.GBStatus.set("vibe", {
                state: state || "idle",
                text: text || "Ready",
                busy: state === "running",
                progress: state === "running" ? { indeterminate: true, label: text || "" } : null,
            });
        } else {
            var statusEl = document.querySelector("#window-vibe .window-statusbar-status");
            if (statusEl) {
                statusEl.textContent = text || "";
                statusEl.dataset.state = state || "idle";
            }
        }
        document.dispatchEvent(new CustomEvent("gb:vibe-status", {
            detail: { status: text || "", state: state || "idle" },
        }));
    }

    function updateBusyLabel(text) {
        var ov = document.getElementById("vibeBusyOverlay");
        if (!ov) return;
        var b = ov.querySelector("b");
        if (b) b.textContent = text || "";
    }

    /* Raise the Vibe window to the topmost z-order so the busy overlay and
       the status-bar progress are actually on screen while a run boots. */
    function bringVibeToFront() {
        var mgr = wm();
        if (!mgr) return;
        try {
            if (typeof mgr.focus === "function") { mgr.focus("vibe"); return; }
        } catch (e) { /* fall through to focusWindow */ }
        if (typeof mgr.focusWindow === "function") mgr.focusWindow("vibe");
    }

    /* Resolve the selected project's kind ('bot' | 'website' | 'apps'...) for
       the Run flow (mirrors the build()-scoped selectedProjectKind). */
    function resolvedProjectKind() {
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
