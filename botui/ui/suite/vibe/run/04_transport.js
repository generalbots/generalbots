/* Vibe Run Dock — split from vibe/vibe-run.js for #1731 */

    /* ------------------------------------------------- runFocus */

    function runFocus(runId) {
        if (!runId) return;
        state.runId = String(runId);
        state.run = null;
        state.metrics = null;
        state.events = [];
        state.progress = 0;
        state.phase = "planning";
        state.stageDone = {};
        state.stageActive = null;
        var card = q("vibeRunCard");
        if (card) card.classList.remove("visible");
        var sources = q("vibeSourcesSection");
        if (sources) sources.style.display = "none";
        setText("vibeRunId", "RUN " + shortRunId(state.runId));
        renderRunCard();
        startPolling();
        startTicker();
        loadRuns();
        loadSessions();
    }

    /* ------------------------------------------------- run create */

    function start(intent, opts) {
        opts = opts || {};
        uiMsg("🔄 Starting Vibe run: " + intent.substring(0, 80) + "…");
        // Attach the currently selected project (if any) so the agent edits
        // the right workspace and the deploy pipeline targets the right VM.
        var pid = null;
        if (typeof currentProjectId !== "undefined" && currentProjectId) pid = currentProjectId;
        var pname = null;
        if (typeof currentProject !== "undefined" && currentProject) pname = currentProject;
        var activeBotName =
            (typeof vibeBotName !== "undefined" && vibeBotName) ||
            window.__INITIAL_BOT_NAME__ ||
            "default";
        var body = {
            intent: intent,
            // The default bot is resolved server-side from the authenticated
            // session. Sending its UUID as an explicit target incorrectly
            // triggers cross-bot RBAC and blocks legitimate project edits.
            bot_id: activeBotName !== "default" && typeof vibeBotId !== "undefined" && vibeBotId
                ? vibeBotId
                : null,
            use_case: state.useCase,
            // Approval gates removed — everything runs automatically; kept
            // for wire compatibility with the server's run config schema.
            auto_approve: true,
            // "deploy" routes the run through the deploy pipeline (publish to
            // production); omitted/other values test in development. Run =
            // dev, Deploy = prod — the only two paths.
            pipeline_mode: opts.pipeline_mode || null,
            project_id: pid,
            project_name: pname,
        };
        return api("/api/vibe/run", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(body),
        }).then(function (data) {
            if (data.success && data.run_id) {
                // The dock state chip may not exist yet (the Run Dock window
                // is only opened after the run is created). Guard it — an
                // unguarded null deref here rejected the promise and turned
                // a successful run creation into "START FAILED".
                var dockState = q("vibeRunDockState");
                if (dockState) {
                    dockState.textContent = "ACTIVE";
                    dockState.className = "vibe-chip state-running";
                }
                runFocus(data.run_id);
                return { ok: true, run_id: data.run_id };
            }
            var err = data.error || ("HTTP " + (data.http_status || "?"));
            uiMsg("⚠️ Vibe run API unavailable (" + err + ") — falling back to autotask flow.");
            var dockIdle = q("vibeRunDockState");
            if (dockIdle) {
                dockIdle.textContent = "IDLE";
                dockIdle.className = "vibe-chip state-pending";
            }
            throw new Error(err);
        });
    }

    function retry(fn, key) {
        var n = (retry.count[key] = (retry.count[key] || 0) + 1);
        if (n < 30) setTimeout(fn, 1000);
    }
    retry.count = {};

    /* ------------------------------------------------- preview */

    /* #1192 — run the project's OWN custom app from its workspace instead of a
       bundled template. Falls back to the deployed preview URL when the
       workspace has no source. */
    function runWorkspaceServeUrl(projectId) {
        if (!projectId) return Promise.resolve(null);
        return api("/api/vibe/projects/" + encodeURIComponent(projectId) + "/files")
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

    // Handle of the popup tab currently showing the live project preview.
    // Stop must close it back (pressing ■ tears down what ▶ launched).
    var previewWindowRef = null;

    function runClosePreview() {
        if (previewWindowRef && !previewWindowRef.closed) {
            try { previewWindowRef.close(); } catch (e) { /* cross-origin: ignore */ }
            uiMsg("⛔ Preview closed.");
        }
        previewWindowRef = null;
    }

    function previewProject() {
        var projectId = typeof currentProjectId !== "undefined" ? currentProjectId : null;
        if (!projectId) {
            uiMsg("⚠️ Select a project before opening Preview App.");
            return;
        }
        // Open synchronously so browser popup blockers do not swallow the
        // preview tab while the authenticated API request is in flight.
        previewWindowRef = window.open("about:blank", "_blank");
        var previewWindow = previewWindowRef;
        if (!previewWindow) {
            uiMsg("⚠️ Allow pop-ups to open the project preview.");
            return;
        }
        previewWindow.document.body.innerHTML = "<p style='font-family:system-ui;padding:24px'>Resolving project preview…</p>";
        runWorkspaceServeUrl(projectId).then(function (url) {
            if (url) return url;
            return api("/api/vibe/projects/" + encodeURIComponent(projectId)).then(function (projectData) {
                var project = projectData && projectData.project;
                var env = (project && project.environment) || "development";
                return api("/api/vibe/projects/" + encodeURIComponent(projectId) + "/preview?env=" + encodeURIComponent(env));
            }).then(function (data) {
                if (typeof data === "string") return data;
                var payload = data && data.data ? data.data : data;
                var url = payload && payload.preview_url;
                if (!url || (String(url).indexOf("http://") !== 0 && String(url).indexOf("https://") !== 0)) {
                    throw new Error("No live preview is available yet. Deploy the project first.");
                }
                return url;
            });
        }).then(function (url) {
            previewWindow.location.href = url;
            uiMsg("🌐 Preview opened: " + url);
            return null;
        }).catch(function (err) {
            previewWindow.close();
            uiMsg("⚠️ Preview unavailable: " + err);
        });
    }

    // #run-done — a completed run posts a system line in the Runner Log
    // with a CLICKABLE deeplink: clicking reopens the app in the Browser
    // window (non-deploy) or the production tab (deploy) without re-running
    // anything. The handler lives in wire() as a delegated listener.
    // Guarded by sessionStorage per run: the WS onProgress path and the
    // pollRun path can both observe the completion — only the first posts.
    function showDoneDeeplink(data, isDeploy, runId) {
        if (typeof vibeSafeMsg !== "function") return;
        var guardKey = "vibeDoneLink:" + (runId || state.runId || "anon");
        if (window.sessionStorage && sessionStorage.getItem(guardKey)) return;
        if (window.sessionStorage) sessionStorage.setItem(guardKey, "1");
        var pid = String(data.project_id || "");
        var name = String(data.project_name || "");
        var label = name ? "@" + name : "the app";
        // vibeRunEsc() escapes < > & but not quotes — attribute values need "" safe too.
        var attr = vibeRunEsc(pid).replace(/"/g, "&quot;");
        var nameAttr = vibeRunEsc(name).replace(/"/g, "&quot;");
        var msg;
        if (isDeploy) {
            msg = "✅ Run completed — <a href=\"#\" data-vibe-open-prod=\"" + attr +
                "\" data-vibe-open-prod-name=\"" + nameAttr +
                "\" style=\"color:var(--accent,#84d669);text-decoration:underline\">open " + vibeRunEsc(label) +
                "</a> in a new tab (production).";
        } else {
            msg = "✅ Run completed — <a href=\"#\" data-vibe-open-app=\"" + attr +
                "\" style=\"color:var(--accent,#84d669);text-decoration:underline\">open " + vibeRunEsc(label) +
                "</a> in the Browser window (click to refresh).";
        }
        vibeSafeMsg("system", msg);
    }

    // Opens the deployed app (https://{app}.{platform-domain}) in a fresh
    // tab. Falls back to the backend preview URL (env=production) when the
    // project name is unknown. Guarded by sessionStorage per run so the
    // Deploy dialog poller and this WS handler never open two tabs.
    function openProdTab(projectName, projectId, runId) {
        var guardKey = "vibeDeployTabOpened:" + (runId || "anon");
        if (window.sessionStorage && sessionStorage.getItem(guardKey)) return;
        var slug = String(projectName || "").toLowerCase().replace(/[^a-z0-9-]+/g, "-").replace(/^-+|-+$/g, "");
        var host = (window.location.hostname || "").split(".").slice(-2).join(".");
        var direct = slug ? "https://" + slug + "." + host + "/" : null;
        var w = window.open("about:blank", "_blank");
        if (!w) {
            if (typeof vibeSafeMsg === "function") {
                vibeSafeMsg("system", "⚠️ Deploy finished — allow pop-ups to open " + (direct || "your app") + ".");
            }
            return;
        }
        if (window.sessionStorage) sessionStorage.setItem(guardKey, "1");
        w.document.body.innerHTML = "<p style='font-family:system-ui;padding:24px'>Opening deployed app…</p>";
        if (direct) { w.location.href = direct; return; }
        if (!projectId) {
            w.close();
            if (typeof vibeSafeMsg === "function") {
                vibeSafeMsg("system", "✅ Deploy finished — open your app at its public URL.");
            }
            return;
        }
        api("/api/vibe/projects/" + encodeURIComponent(projectId) + "/preview?env=production")
            .then(function (data) {
                var payload = data && data.data ? data.data : data;
                var url = payload && payload.preview_url;
                if (url && /^https?:/i.test(String(url))) { w.location.href = url; return; }
                w.close();
                if (typeof vibeSafeMsg === "function") {
                    vibeSafeMsg("system", "✅ Deploy finished — no public URL reported for the app.");
                }
            })
            .catch(function () {
                w.close();
                if (typeof vibeSafeMsg === "function") {
                    vibeSafeMsg("system", "✅ Deploy finished — could not resolve the public URL.");
                }
            });
    }
