/* Vibe Shell toolbar — split from vibe-shell/10_toolbar.js for #1731 */

    /* #1271 — Chat button opens the shared Chat window as a NEW
       conversation (no session param = fresh) and pre-fills the input with a
       directive so the message is routed to the app currently running in
       vibe. The pre-fill is the RUNNING APP'S NAME (@qa-flow), never a long
       botbook file path — the user must see the app name in the input, not a
       path to a .md file. Falls back to the botbook directive only when
       no project is selected.
       #1504 — bot-kind projects open the Chat window in two-env mode: the
       window renders {NAME} (TEST) / {NAME} (PROD) conversation tabs bound
       to the `{bot}-test` / `{bot}` bot identities. */
    var CHAT_BOOK_DIRECTIVE =
        "@botbook/src/10-configuration-deployment/config-csv.md";
    function openChat() {
        var pid = S.projectId();
        var name = S.projectName();
        var kind = String(
            (typeof window.currentProjectKind !== "undefined" && window.currentProjectKind) ||
            (function () { try { return localStorage.getItem("gb-vibe-project-kind") || ""; } catch (e) { return ""; } })()
        ).toLowerCase();
        if (pid && name && kind === "bot") {
            openBotChat(name, null);
            return;
        }
        var message =
            pid && name && String(name) !== "vibe"
                ? "@" + name
                : CHAT_BOOK_DIRECTIVE;
        openSharedApp("chat", { message: message });
    }

    /* #1504 — open the Chat window for a bot project. `env` selects the
       conversation: null/"test" → TEST tab (default), "production" → PROD.
       The tab strip itself is rendered by the Chat window module when the
       deep-link carries botTest+botProd+botEnv (vibe-bot-env.js). */
    function openBotChat(projectName, env) {
        var slug = String(projectName || "").toLowerCase().replace(/\s+/g, "-");
        var params = {
            bot: env === "production" ? slug : slug + "-test",
            botTest: slug + "-test",
            botProd: slug,
            botEnv: env || "test",
            botLabel: projectName,
        };
        if (env === "production") {
            params.bot = slug;
        }
        openSharedApp("chat", params);
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
        var kind = resolvedProjectKind();
        var isBot = kind === "bot";

        // #1409 follow-up — a project takes time to boot. Raise Vibe to the
        // topmost window, freeze the desktop behind a spinner, and narrate
        // the stages in the Vibe window status bar until the app is up.
        // Then open the right surface: a bot project opens its Chat window on
        // the new conversation, a website/app project opens the Browser.
        bringVibeToFront();
        setBusy(true, "Starting " + projectName + "…");
        toolbarSetVibeStatus("Starting " + projectName + " — provisioning the dev VM…", "running");

        var finished = false;
        function endBusy() { if (!finished) { finished = true; setBusy(false); } }

        // Run is a deterministic "start the app on the dev VM" action: it
        // pushes the workspace into the container, restarts the node service,
        // and reveals the app. It must NOT invoke the agent loop
        // (VibeTransport.play), which spends LLM tool calls running/verifying
        // the project — that belongs to the Chat agent, not the Run button.
        startDevVm(projectId)
            .then(function (vm) {
                if (isBot) {
                    updateBusyLabel("Opening the bot window (TEST)…");
                    toolbarSetVibeStatus(projectName + " is running — opening the TEST conversation…", "running");
                    // #1504 — Run recompiles the {bot}-test twin server-side
                    // and opens the Chat window on the TEST tab.
                    D.api("/api/vibe/projects/" + encodeURIComponent(projectId) + "/bot/run-test", { method: "POST" })
                        .catch(function () { /* the chat still opens best-effort */ })
                        .finally(function () {
                            openBotChat(projectName, "test");
                            setRunVisual(true);
                            endBusy();
                            flashHint("BOT " + String(projectName).toUpperCase() + " (TEST) IS RUNNING");
                        });
                    return;
                }
                updateBusyLabel("Opening the app in the browser…");
                openBrowser(vm.url);
                setRunVisual(true);
                toolbarSetVibeStatus("Running " + projectName + " on the dev VM", "running");
                endBusy();
                flashHint("RUNNING " + String(projectName).toUpperCase() + " ON THE DEV VM");
            })
            .catch(function () {
                if (isBot) {
                    // The TEST conversation still opens (best effort); the
                    // twin row exists regardless of the dev service.
                    openBotChat(projectName, "test");
                    setRunVisual(true);
                    toolbarSetVibeStatus("TEST conversation opened (dev VM unavailable)", "running");
                    endBusy();
                    flashHint("NO DEV VM — OPENED TEST CONVERSATION");
                    return;
                }
                return workspaceServeUrl(projectId)
                    .then(function (url) {
                        if (url) return url;
                        return resolvePreviewUrl(projectId);
                    })
                    .then(function (url) {
                        updateBusyLabel("Opening the static preview…");
                        openBrowser(url);
                        setRunVisual(true);
                        toolbarSetVibeStatus("Running " + projectName + " (static preview — no VM)", "running");
                        endBusy();
                        flashHint("RUNNING " + String(projectName).toUpperCase() + " (STATIC PREVIEW — NO VM)");
                    })
                    .catch(function (err) {
                        setRunVisual(false);
                        openBrowser(null);
                        toolbarSetVibeStatus("Not running — deploy the project to view it", "failed");
                        endBusy();
                        flashHint((err && err.message ? err.message : "No preview available") + " — deploy the project to see your app");
                    });
            });
    }

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
