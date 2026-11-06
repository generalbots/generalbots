/* Vibe Deploy dialog — split from vibe/vibe-dialog-deploy.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__deploy;
    var D = window.VibeDialogs;
    // Polls the deploy run until it reaches a terminal state, then opens the
    // published app in a NEW browser tab (popup-friendly about:blank flow).
    function waitDeployFinished(runId, attempt) {
        if (!runId || attempt > 120) {
            return; // 10 min cap — the Run Dock still shows live progress.
        }
        D.api("/api/vibe/run/" + encodeURIComponent(runId)).then(function (data) {
            // `runState` (not `state`) — a local `state` here would shadow the
            // dialog state and break the #1267 projectName lookup below.
            var runState = data && (data.state || (data.data && data.data.state));
            if (runState === "completed") {
                var grid = document.getElementById("vibeDeployMain");
                if (grid) {
                    grid.innerHTML = '<div class="vibe-empty">✅ Deployed — opening <b>' +
                        D.esc(M.state.projectName || "project") + "</b>…</div>";
                }
                // #1504 — bot projects open the Chat window on the PROD tab
                // (the bot is a suite conversation, not a browser page);
                // websites/apps keep opening the deployed URL in a new tab.
                if (M.isBotProject()) {
                    M.promoteBotProd();
                } else {
                    M.openAppTab("production");
                }
                setTimeout(M.loadHistory, 2500);
            } else if (runState === "failed" || runState === "cancelled" || runState === "aborted") {
                var gf = document.getElementById("vibeDeployMain");
                if (gf) {
                    gf.innerHTML = '<div class="vibe-empty">Deploy ' + D.esc(runState) +
                        ": " + D.esc((data && data.error) || "see Runner Log for details.") + "</div>";
                }
            } else {
                setTimeout(function () { M.waitDeployFinished(runId, attempt + 1); }, 5000);
            }
        }).catch(function () {
            setTimeout(function () { M.waitDeployFinished(runId, attempt + 1); }, 5000);
        });
    }

    // #1504 — bot-kind project detection for the deploy-complete flow: the
    // toolbar state carries the project kind (same helper the shell uses).
    function isBotProject() {
        try {
            var kind = window.currentProjectKind || localStorage.getItem("gb-vibe-project-kind") || "";
            return String(kind).toLowerCase() === "bot";
        } catch (e) { return false; }
    }

    // #1504 — bot PROD completion: promote the TEST release into the PROD
    // bot layout server-side, then open the Chat window on the PROD tab.
    function promoteBotProd() {
        D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/bot/deploy-prod", {
            method: "POST",
        }).then(function () {
            var slug = String(M.state.projectName || "").toLowerCase().replace(/\s+/g, "-");
            if (window.VibeShell && window.VibeShell.toolbar && window.VibeShell.toolbar.openBotChat) {
                window.VibeShell.toolbar.openBotChat(M.state.projectName || slug, "production");
            } else {
                openSharedApp("chat", {
                    bot: slug,
                    botTest: slug + "-test",
                    botProd: slug,
                    botEnv: "production",
                    botLabel: M.state.projectName || slug,
                });
            }
        }).catch(function (err) {
            var grid = document.getElementById("vibeDeployMain");
            if (grid) grid.innerHTML = '<div class="vibe-empty">Deploy promote failed: ' + D.esc(err) + "</div>";
        });
    }

    // Opens the app's production URL in a fresh tab: prefer the backend
    // preview URL, fall back to the platform route {repo}.{domain} so a
    // deployed app ALWAYS opens at {app}.generalbots.org.
    function openAppTab(envName) {
        var w = window.open("about:blank", "_blank");
        if (!w) {
            var grid = document.getElementById("vibeDeployMain");
            if (grid) grid.innerHTML = '<div class="vibe-empty">Allow pop-ups to open the deployed app.</div>';
            return;
        }
        w.document.body.innerHTML = "<p style='font-family:system-ui;padding:24px'>Opening deployed app…</p>";
        var open = function (url) {
            if (url && (String(url).indexOf("http://") === 0 || String(url).indexOf("https://") === 0)) {
                w.location.href = url;
            } else {
                w.close();
                var grid = document.getElementById("vibeDeployMain");
                if (grid) grid.innerHTML = '<div class="vibe-empty">Deployed, but no live URL was reported.</div>';
            }
        };
        D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/preview?env=" + encodeURIComponent(envName))
            .then(function (data) {
                var payload = data && data.data ? data.data : data;
                if (payload && payload.preview_url) { open(payload.preview_url); return; }
                var slug = String(M.state.projectName || "").toLowerCase().replace(/[^a-z0-9-]+/g, "-").replace(/^-+|-+$/g, "");
                var host = (window.location.hostname || "").split(".").slice(-2).join(".");
                open("https://" + encodeURIComponent(slug) + "." + host + "/");
            })
            .catch(function () {
                var slug = String(M.state.projectName || "").toLowerCase().replace(/[^a-z0-9-]+/g, "-").replace(/^-+|-+$/g, "");
                var host = (window.location.hostname || "").split(".").slice(-2).join(".");
                open("https://" + encodeURIComponent(slug) + "." + host + "/");
            });
    }

    M.waitDeployFinished = waitDeployFinished;
    M.isBotProject = isBotProject;
    M.promoteBotProd = promoteBotProd;
    M.openAppTab = openAppTab;
})(window.VibeDialogs);
