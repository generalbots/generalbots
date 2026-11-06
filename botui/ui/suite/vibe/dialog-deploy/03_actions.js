/* Vibe Deploy dialog — split from vibe/vibe-dialog-deploy.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__deploy;
    var D = window.VibeDialogs;
    function probeEnv() {
        if (!M.state.projectId) return;
        // Probe the project's deployed environment (production for a
        // published app) instead of hardcoding "development" — the project
        // registry stores the env per project (e.g. calculator → production).
        var proj = M.state.projects.find(function (x) { return x.id === M.state.projectId; });
        var env = (proj && (proj.environment || proj.env)) || "production";
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Probing environment (' + D.esc(env) + ")...</div>";
        D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/envs/" + encodeURIComponent(env) + "/probe", {
            method: "POST",
            body: {},
        }).then(function (data) {
            var probe = (data && data.probe) || (data && data.data && data.data.probe) || (data && data.data && data.data.data && data.data.data.probe);
            var url = probe && probe.url;
            var html = "";
            if (url) {
                // The whole point of deploy: a clickable link to the running app.
                html += '<div class="vibe-card" style="margin-bottom:10px;padding:12px;border:1px solid var(--accent,#84d669);">' +
                    '<div style="font-size:11px;color:var(--text-muted);margin-bottom:6px;">🚀 OPEN YOUR APP</div>' +
                    '<a href="' + D.esc(url) + '" target="_blank" rel="noopener" style="font-size:14px;font-weight:700;color:var(--accent,#84d669);text-decoration:none;">' +
                    D.esc(url) + " ↗</a>" +
                    '<div style="font-size:10px;color:var(--text-muted);margin-top:6px;">env ' + D.esc(env) +
                    (probe.http_code ? " · http " + probe.http_code : "") +
                    (probe.ok ? " · UP" : " · down") + "</div></div>";
            } else if (probe && probe.running === false) {
                html += '<div class="vibe-empty">Container not running — launch the deploy pipeline.</div>';
            }
            html += '<div class="vibe-diff"><pre>' + D.esc(JSON.stringify(data || {}, null, 2)) + "</pre></div>";
            if (grid) grid.innerHTML = html;
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty">Probe error: ' + D.esc(err) + "</div>";
        });
    }

    // Every website/python project keeps TWO websites: the working copy at
    // {slug}-test.{domain} (where in-flight changes land) and the public page
    // at {slug}.{domain}. Promote copies the tested release — the exact
    // payload that was served and verified as the test site — onto the public
    // slug, so nobody has to edit the live page directly.
    function promoteSite() {
        if (!M.state.projectId) return;
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Promoting the tested release to the public site…</div>';
        D.api(
            "/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/site/promote",
            { method: "POST", body: {} }
        ).then(function (data) {
            var url = data && (data.url || (data.data && data.data.url));
            if (grid) {
                grid.innerHTML = '<div class="vibe-card" style="margin-bottom:10px;padding:12px;border:1px solid var(--accent,#84d669);">' +
                    '<div style="font-size:11px;color:var(--text-muted);margin-bottom:6px;">✅ PUBLIC SITE UPDATED</div>' +
                    (url
                        ? '<a href="' + D.esc(url) + '" target="_blank" rel="noopener" style="font-size:14px;font-weight:700;color:var(--accent,#84d669);text-decoration:none;">' + D.esc(url) + " ↗</a>"
                        : "<div>Promoted.</div>") +
                    "</div>";
            }
            M.loadHistory();
            M.loadSecurity();
        }).catch(function (err) {
            var message = err && err.message ? err.message : String(err);
            if (grid) {
                grid.innerHTML = '<div class="vibe-empty">Promote unavailable: ' + D.esc(message) +
                    ' — publish the project to its test site first.</div>';
            }
        });
    }

    function previewApp() {
        if (!M.state.projectId) return;
        var previewWindow = window.open("about:blank", "_blank");
        if (!previewWindow) {
            var grid = document.getElementById("vibeDeployMain");
            if (grid) grid.innerHTML = '<div class="vibe-empty">Allow pop-ups to open the app preview.</div>';
            return;
        }
        previewWindow.document.body.innerHTML = "<p style='font-family:system-ui;padding:24px'>Resolving project preview…</p>";
        D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId)).then(function (projectData) {
            var project = projectData && projectData.project;
            var env = (project && (project.environment || project.env)) || "development";
            return D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/preview?env=" + encodeURIComponent(env));
        }).then(function (data) {
            var payload = data && data.data ? data.data : data;
            var url = payload && payload.preview_url;
            if (!url || (String(url).indexOf("http://") !== 0 && String(url).indexOf("https://") !== 0)) {
                previewWindow.close();
                var grid = document.getElementById("vibeDeployMain");
                if (grid) grid.innerHTML = '<div class="vibe-empty">No live URL yet. Deploy the project first.</div>';
                return;
            }
            previewWindow.location.href = url;
        }).catch(function (err) {
            previewWindow.close();
            var grid = document.getElementById("vibeDeployMain");
            if (grid) grid.innerHTML = '<div class="vibe-empty">Preview error: ' + D.esc(err) + "</div>";
        });
    }

    function deployPipeline() {
        if (!M.state.projectId) return;
        var proj = M.state.projects.find(function (x) { return x.id === M.state.projectId; });
        var projName = proj ? proj.name : "project";
        // #1267 — keep the name on the dialog state: the async poller and
        // tab-opener below run after this function returned and previously
        // referenced the function-local `projName` (ReferenceError).
        M.state.projectName = projName;
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Launching deploy pipeline (fully automated)...</div>';
        D.api("/api/vibe/run", {
            method: "POST",
            body: {
                intent: "Deploy " + projName,
                use_case: "software_development",
                pipeline_mode: "deploy",
                auto_approve: false,
                project_id: M.state.projectId,
                project_name: projName,
            },
        }).then(function (data) {
            if (data && data.success) {
                if (grid) {
                    grid.innerHTML = '<div class="vibe-empty">Deploy pipeline started for <b>' +
                        D.esc(projName) + "</b> — approve stages in the Run Dock.</div>";
                }
                setTimeout(M.loadHistory, 8000);
                // Auto-open the deployed app in a new tab once the pipeline
                // finishes: poll the run, then open the production URL
                // (https://{app}.{platform-domain}) in a fresh tab.
                M.waitDeployFinished(data.run_id, 0);
            } else {
                if (grid) grid.innerHTML = '<div class="vibe-empty">Deploy start failed: ' + D.esc((data && data.error) || "unknown") + "</div>";
            }
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty">Deploy error: ' + D.esc(err) + "</div>";
        });
    }

    M.probeEnv = probeEnv;
    M.promoteSite = promoteSite;
    M.previewApp = previewApp;
    M.deployPipeline = deployPipeline;
})(window.VibeDialogs);
