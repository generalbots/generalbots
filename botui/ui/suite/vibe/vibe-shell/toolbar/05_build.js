/* Vibe Shell toolbar — split from vibe-shell/10_toolbar.js for #1731 */

    function build() {
        if (document.getElementById("vibeShellToolbar")) return;
        var container = document.getElementById("vibeWindow");
        if (!container) return;

        var bar = toolbarEl("div", "vibe-shell-toolbar vibe-shell-tb-grid");
        bar.id = "vibeShellToolbar";
        bar.setAttribute("role", "toolbar");
        bar.setAttribute("aria-label", "Vibe commands");

        /* ── RUN + DEPLOY — the two primaries, together at LEFT ─── */
        var runGroup = toolbarEl("div", "vibe-shell-tb-group vibe-shell-tb-run-group");
        runGroup.appendChild(buildButton("Run", "Run", openPreview, "vibe-shell-tb-run"));
        bar.appendChild(runGroup);

        var deployGroup = toolbarEl("div", "vibe-shell-tb-group vibe-shell-tb-deploy-group");
        deployGroup.appendChild(buildButton("Deploy", "Deploy", deployProject, "vibe-shell-tb-deploy"));
        bar.appendChild(deployGroup);

        /* ── Combos — two rows: Project on top, Branch below ────── */
        var selectors = toolbarEl("div", "vibe-shell-tb-group vibe-shell-tb-selectors");

        var projWrap = toolbarEl("label", "vibe-shell-tb-field");
        projWrap.appendChild(toolbarEl("span", "vibe-shell-tb-field-label", "Project"));
        var sel = toolbarEl("select", "vibe-shell-project-select");
        sel.id = "vibeShellProjectSelect";
        sel.title = "Active project";
        sel.setAttribute("aria-label", "Active project");
        sel.disabled = true;
        sel.appendChild(toolbarEl("option", null, "Loading…"));
        sel.addEventListener("change", onProjectChange);
        projWrap.appendChild(sel);
        selectors.appendChild(projWrap);

        var brWrap = toolbarEl("label", "vibe-shell-tb-field");
        brWrap.appendChild(toolbarEl("span", "vibe-shell-tb-field-label", "Branch"));
        var brSel = document.createElement("select");
        brSel.id = "vibeShellBranchSelect";
        brSel.className = "vibe-shell-project-select vibe-shell-branch-select";
        brSel.title = "Active branch";
        brSel.setAttribute("aria-label", "Active branch");
        brSel.disabled = true;
        brSel.appendChild(toolbarEl("option", null, "—"));
        brSel.addEventListener("change", onBranchChange);
        brWrap.appendChild(brSel);
        selectors.appendChild(brWrap);
        bar.appendChild(selectors);

    /* ── Window commands — 3 rows × 4 columns (#1383 review): a single
       full-height separator spans the three lines between column groups ── */
        var cmds = toolbarEl("div", "vibe-shell-tb-group vibe-shell-tb-cmds");
        // 12 commands in a 3×4 grid — exactly the revised issue sketch:
        //   row 1: New Project | Browser        | Editor         | Canvas
        //   row 2: Chat        | Terminal       | Database       | Knowledge Graph
        //   row 3: Properties  | Run Logger     | Source Control | Members
        var editorPid = S.projectId();
        var defs = [
            ["New Project", "New Project", function () { if (window.VibeNewProject) window.VibeNewProject.open(); else if (window.VibeWindows) window.VibeWindows.openNewProject(); }, "vibe-shell-tb-new"],
            // #1504 — bot-kind projects get a "Bot" button (opens the bot's
            // TEST conversation); website/app keep the Browser preview.
            ["Browser", "Browser", function () {
                var pid = S.projectId();
                var name = S.projectName();
                if (pid && name && selectedProjectKind() === "bot") {
                    openBotChat(name, "test");
                    return;
                }
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
            var sep = toolbarEl("span", "vibe-shell-tb-sep vibe-shell-tb-sep3");
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
            // #1504 — the second command slot is "Bot" for bot projects
            // (opens the TEST conversation) and "Browser" otherwise.
            Array.prototype.forEach.call(bar.querySelectorAll(".vibe-shell-tb-btn"), function (b) {
                var labelEl = b.querySelector(".vibe-shell-tb-label");
                if (!labelEl) return;
                if (labelEl.textContent === "Browser" && kind === "bot") {
                    labelEl.textContent = "Bot";
                    b.title = "Open the bot conversation (TEST)";
                } else if (labelEl.textContent === "Bot" && kind !== "bot") {
                    labelEl.textContent = "Browser";
                    b.title = "Browser";
                }
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
        var closeGroup = toolbarEl("div", "vibe-shell-tb-group vibe-shell-tb-close-group");
        var closeBtn = toolbarEl("button", "vibe-shell-tb-btn vibe-shell-tb-closeall");
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
            if (!src) return;
            var text = String(src.textContent || "").trim();
            if (!text) return;
            var state = /running|execut/i.test(text) ? "running"
                : /paus|approv/i.test(text) ? "paused"
                : /fail|error|stop/i.test(text) ? "failed"
                : "idle";
            // #1849 — the footer is rendered by the shared status contract.
            if (window.GBStatus && typeof window.GBStatus.set === "function") {
                window.GBStatus.set("vibe", {
                    state: state,
                    text: text,
                    busy: state === "running",
                    progress: state === "running" ? { indeterminate: true, label: text } : null,
                });
            } else {
                var statusEl = document.querySelector("#window-vibe .window-statusbar-status");
                if (!statusEl) return;
                statusEl.textContent = text;
                statusEl.dataset.state = state;
            }
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
        // #1408 — when a Vibe agent run finishes, refresh every open
        // integrated app: re-target the Browser preview to the run's project
        // (dev VM first) and signal Editor/Canvas/Database to reload their
        // project-scoped content. announceRunFinished (vibe-run.js) dedupes.
        document.addEventListener("gb:vibe-run-finished", function (e) {
            var detail = (e && e.detail) || {};
            if (detail.project && typeof openProjectApp === "function") {
                openProjectApp(detail.project);
            }
            document.dispatchEvent(new CustomEvent("gb:vibe-app-refresh", { detail: detail }));
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
        openBotChat: openBotChat,
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
