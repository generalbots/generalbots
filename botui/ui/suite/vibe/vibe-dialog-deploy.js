/**
 * Vibe Deploy dialog (#806 rewrite) — real deployment surface.
 * Sidebar: project selector + deployments history
 * (/api/vibe/projects/:id/deployments). Main: environment status,
 * probe/restart (envs/:env/probe|restart) and deploy pipeline launch
 * (POST /api/vibe/run with pipeline_mode=deploy).
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { projects: [], projectId: null, history: [] };

    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        var head = D.el("div", "vibe-dialog-title");
        head.textContent = "DEPLOYMENTS";
        head.style.padding = "10px";

        var projBar = D.el("div", "vibe-commit-box");
        var sel = D.el("select", "vibe-select");
        sel.id = "vibeDeployProject";
        sel.innerHTML = "<option>no project</option>";
        sel.style.width = "100%";
        sel.addEventListener("change", function () {
            state.projectId = sel.value || null;
            loadHistory();
        });
        projBar.innerHTML = "<div style='font-size:10px;color:var(--text-muted);margin-bottom:6px'>project</div>";
        projBar.appendChild(sel);

        var list = D.el("div", "vibe-list");
        list.id = "vibeDeployList";
        list.innerHTML = '<div class="vibe-empty">Select a project to see deployments.</div>';

        box.appendChild(head);
        box.appendChild(projBar);
        box.appendChild(list);
        return box;
    }

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";

        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var label = D.el("span", "vibe-status info", "no project");
        label.id = "vibeDeployLabel";
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var probe = D.el("button", "vibe-btn", "🔍 Probe env");
        probe.addEventListener("click", probeEnv);
        var deployBtn = D.el("button", "vibe-btn primary", "🚀 Deploy pipeline");
        deployBtn.addEventListener("click", deployPipeline);
        toolbar.appendChild(label);
        toolbar.appendChild(spacer);
        toolbar.appendChild(probe);
        toolbar.appendChild(deployBtn);

        var grid = D.el("div", "vibe-grid");
        grid.id = "vibeDeployMain";
        grid.innerHTML = '<div class="vibe-empty">Pick a project. Deploy runs the intents→build→test→' +
            "publish pipeline (approval-gated stages) on the backend.</div>";

        box.appendChild(toolbar);
        box.appendChild(grid);
        return box;
    }

    function loadProjects() {
        D.api("/api/vibe/projects").then(function (data) {
            state.projects = (data && data.projects) || [];
            var sel = document.getElementById("vibeDeployProject");
            if (!sel) return;
            sel.innerHTML = "";
            if (!state.projects.length) {
                sel.innerHTML = "<option>no projects yet</option>";
                return;
            }
            state.projects.forEach(function (p) {
                var opt = document.createElement("option");
                opt.value = p.id;
                opt.textContent = p.name || String(p.id).substring(0, 8);
                sel.appendChild(opt);
            });
            var wanted = state.projectId;
            if (wanted && document.querySelector('#vibeDeployProject option[value="' + wanted + '"]')) {
                sel.value = wanted;
            } else if (typeof currentProjectId !== "undefined" && currentProjectId) {
                sel.value = currentProjectId;
            }
            state.projectId = sel.value || null;
            var label = document.getElementById("vibeDeployLabel");
            if (label && state.projectId) {
                label.textContent = "project " + state.projectId.substring(0, 8);
                label.className = "vibe-status ok";
            }
            if (state.projectId) loadHistory();
        }).catch(function () {
            var sel = document.getElementById("vibeDeployProject");
            if (sel) sel.innerHTML = "<option>projects API unavailable</option>";
        });
    }

    function loadHistory() {
        if (!state.projectId) return;
        var list = document.getElementById("vibeDeployList");
        var grid = document.getElementById("vibeDeployMain");
        if (list) list.innerHTML = '<div class="vibe-empty">Loading deployments...</div>';
        D.api("/api/vibe/projects/" + encodeURIComponent(state.projectId) + "/deployments").then(function (data) {
            state.history = (data && (data.deployments || data.history)) || [];
            if (list) {
                if (!state.history.length) {
                    list.innerHTML = '<div class="vibe-empty">No deployments yet.</div>';
                } else {
                    list.innerHTML = "";
                    state.history.forEach(function (h, i) {
                        var row = D.el("div", "vibe-list-item");
                        var env = h.env || "—";
                        var st = h.status || h.state || "?";
                        var cls = String(st).toLowerCase() === "active" ? "ok" :
                            String(st).toLowerCase() === "failed" ? "err" : "info";
                        row.innerHTML = "<span>" + D.esc(env) + " #" + (state.history.length - i) + "</span>" +
                            '<span class="vibe-status ' + cls + '">' + D.esc(st) + "</span>";
                        list.appendChild(row);
                    });
                }
            }
            renderHistory();
        }).catch(function () {
            if (list) list.innerHTML = '<div class="vibe-empty">History unavailable (needs VM env).</div>';
            if (grid) grid.innerHTML = '<div class="vibe-empty">Deployment history requires Incus runtime (prod/staging env).</div>';
        });
    }

    function renderHistory() {
        var grid = document.getElementById("vibeDeployMain");
        if (!grid) return;
        if (!state.history.length) {
            grid.innerHTML = '<div class="vibe-empty">No deployment records. Launch the pipeline to create one.</div>';
            return;
        }
        var html = '<table class="vibe-table"><thead><tr><th>#</th><th>Env</th><th>Status</th><th>Run</th><th>When</th></tr></thead><tbody>';
        state.history.forEach(function (h, i) {
            html += "<tr><td>" + (state.history.length - i) + "</td>" +
                "<td>" + D.esc(h.env || "—") + "</td>" +
                "<td>" + D.esc(h.status || h.state || "?") + "</td>" +
                "<td>" + D.esc(String(h.run_id || h.id || "").substring(0, 8)) + "</td>" +
                "<td>" + D.esc(h.created_at || h.ts || "") + "</td></tr>";
        });
        html += "</tbody></table>";
        grid.innerHTML = html;
    }

    function probeEnv() {
        if (!state.projectId) return;
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Probing environment...</div>';
        D.api("/api/vibe/projects/" + encodeURIComponent(state.projectId) + "/envs/development/probe", {
            method: "POST",
            body: {},
        }).then(function (data) {
            if (grid) {
                grid.innerHTML = '<div class="vibe-diff"><pre>' +
                    D.esc(JSON.stringify(data || {}, null, 2)) + "</pre></div>";
            }
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty">Probe error: ' + D.esc(err) + "</div>";
        });
    }

    function deployPipeline() {
        if (!state.projectId) return;
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Launching deploy pipeline (approval-gated)...</div>';
        D.api("/api/vibe/run", {
            method: "POST",
            body: {
                intent: "Deploy project " + state.projectId,
                use_case: "software_development",
                pipeline_mode: "deploy",
                auto_approve: false,
                project_id: state.projectId,
            },
        }).then(function (data) {
            if (data && data.success) {
                if (grid) {
                    grid.innerHTML = '<div class="vibe-empty">Deploy pipeline started: <b>' +
                        D.esc(String(data.run_id).substring(0, 8)) + "</b> — approve stages in the Run Dock.</div>";
                }
                setTimeout(loadHistory, 8000);
            } else {
                if (grid) grid.innerHTML = '<div class="vibe-empty">Deploy start failed: ' + D.esc((data && data.error) || "unknown") + "</div>";
            }
        }).catch(function (err) {
            if (grid) grid.innerHTML = '<div class="vibe-empty">Deploy error: ' + D.esc(err) + "</div>";
        });
    }

    D.register("deploy", {
        build: function (body) {
            body.appendChild(sidebar());
            body.appendChild(main());
            loadProjects();
        },
        teardown: function () {
            state = { projects: [], projectId: null, history: [] };
        },
    });
})();