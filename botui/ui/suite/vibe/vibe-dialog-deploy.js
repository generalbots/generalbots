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
        var preview = D.el("button", "vibe-btn", "🌐 Preview App");
        preview.addEventListener("click", previewApp);
        var deployBtn = D.el("button", "vibe-btn primary", "🚀 Deploy pipeline");
        deployBtn.addEventListener("click", deployPipeline);
        toolbar.appendChild(label);
        toolbar.appendChild(spacer);
        toolbar.appendChild(probe);
        toolbar.appendChild(preview);
        toolbar.appendChild(deployBtn);

        var grid = D.el("div", "vibe-grid");
        grid.id = "vibeDeployMain";
        grid.innerHTML = '<div class="vibe-empty">Pick a project. Deploy runs the intents→build→test→' +
            "publish pipeline (approval-gated stages) on the backend.</div>";

        // ── App Security (access policy per bound domain) ──
        var secBox = D.el("div", "vibe-card");
        secBox.style.margin = "12px 0 0";
        var secHead = D.el("div", "vibe-dialog-title");
        secHead.textContent = "🔐 APP SECURITY";
        secHead.style.padding = "10px";
        var secHint = D.el("div");
        secHint.style.cssText = "font-size:11px;color:var(--text-muted);padding:0 10px 8px;";
        secHint.textContent = "Who can open each domain. Public = anyone · Account = any signed-in user · Allowlist = only listed emails (JWT gate via Caddy).";
        var secBody = D.el("div");
        secBody.id = "vibeSecurityBody";
        secBody.innerHTML = '<div class="vibe-empty">Pick a project to manage its domains.</div>';
        secBox.appendChild(secHead);
        secBox.appendChild(secHint);
        secBox.appendChild(secBody);

        box.appendChild(toolbar);
        box.appendChild(grid);
        box.appendChild(secBox);
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
                var p = state.projects.find(function (x) { return x.id === state.projectId; });
                label.textContent = p ? p.name : "project";
                label.className = "vibe-status ok";
            }
            if (state.projectId) loadHistory();
            if (state.projectId) loadSecurity();
        }).catch(function () {
            var sel = document.getElementById("vibeDeployProject");
            if (sel) sel.innerHTML = "<option>projects API unavailable</option>";
        });
    }

    function loadSecurity() {
        if (!state.projectId) return;
        var body = document.getElementById("vibeSecurityBody");
        if (!body) return;
        body.innerHTML = '<div class="vibe-empty">Loading domains...</div>';
        D.api("/api/vibe/projects/" + encodeURIComponent(state.projectId) + "/domains").then(function (data) {
            var binds = (data && data.binds) || [];
            if (!binds.length) {
                body.innerHTML = '<div class="vibe-empty">No domains bound yet. Bind one via the deploy pipeline (domain/bind).</div>';
                return;
            }
            body.innerHTML = "";
            binds.forEach(function (b) {
                var row = D.el("div", "vibe-sec-row");
                row.style.cssText = "display:flex;flex-wrap:wrap;gap:8px;align-items:center;padding:8px 10px;border-top:1px solid var(--border, #222);";

                var d = D.el("a");
                var proto = location.protocol === "https:" ? "https://" : "http://";
                d.href = proto + (b.domain || "");
                d.target = "_blank";
                d.rel = "noopener";
                d.textContent = (b.verified ? "🔗 " : "⏳ ") + (b.domain || "?");
                d.style.flex = "1";
                d.style.minWidth = "180px";
                d.style.color = b.verified ? "var(--accent, #84d669)" : "var(--text-muted, #888)";
                d.style.textDecoration = b.verified ? "underline" : "none";
                d.title = b.verified ? "Open app (domain " + (b.domain || "") + ")" : "Domain not verified yet";

                var sel = D.el("select", "vibe-select");
                sel.style.minWidth = "130px";
                ["public", "authenticated", "rbac"].forEach(function (v) {
                    var o = document.createElement("option");
                    o.value = v;
                    o.textContent = v === "public" ? "Public" : v === "authenticated" ? "Account" : "Allowlist";
                    sel.appendChild(o);
                });
                sel.value = b.access || "public";

                var emails = D.el("input", "vibe-input");
                emails.type = "text";
                emails.placeholder = "allowed emails (comma separated)";
                emails.value = b.allowed_emails || "";
                emails.style.flex = "1";
                emails.style.minWidth = "220px";
                emails.disabled = sel.value !== "rbac";
                sel.addEventListener("change", function () {
                    emails.disabled = sel.value !== "rbac";
                });

                var save = D.el("button", "vibe-btn", "Save");
                save.addEventListener("click", function () {
                    save.textContent = "Saving…";
                    save.disabled = true;
                    D.api("/api/vibe/domains/" + encodeURIComponent(b.id) + "/access", {
                        method: "PATCH",
                        body: { access: sel.value, allowed_emails: emails.value },
                    }).then(function (res) {
                        save.textContent = "✓ Saved";
                        if (res && res.error) {
                            save.textContent = "✗ " + String(res.error).substring(0, 40);
                        }
                        setTimeout(function () { save.textContent = "Save"; save.disabled = false; }, 2500);
                    }).catch(function (err) {
                        save.textContent = "✗ error";
                        setTimeout(function () { save.textContent = "Save"; save.disabled = false; }, 2500);
                    });
                });

                row.appendChild(d);
                row.appendChild(sel);
                row.appendChild(emails);
                row.appendChild(save);
                body.appendChild(row);
            });
        }).catch(function () {
            body.innerHTML = '<div class="vibe-empty">Domains unavailable (needs VM/Caddy runtime).</div>';
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
            var run = h.run_id || h.id;
            var runLabel = run ? "#" + String(run).substring(0, 4) : "—";
            var when = h.created_at || h.ts || "";
            if (when) when = String(when).replace("T", " ").substring(0, 16);
            html += "<tr><td>" + (state.history.length - i) + "</td>" +
                "<td>" + D.esc(h.env || "—") + "</td>" +
                "<td>" + D.esc(h.status || h.state || "?") + "</td>" +
                "<td>" + D.esc(runLabel) + "</td>" +
                "<td>" + D.esc(when || "—") + "</td></tr>";
        });
        html += "</tbody></table>";
        grid.innerHTML = html;
    }

    function probeEnv() {
        if (!state.projectId) return;
        // Probe the project's deployed environment (production for a
        // published app) instead of hardcoding "development" — the project
        // registry stores the env per project (e.g. calculator → production).
        var proj = state.projects.find(function (x) { return x.id === state.projectId; });
        var env = (proj && (proj.environment || proj.env)) || "production";
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Probing environment (' + D.esc(env) + ")...</div>";
        D.api("/api/vibe/projects/" + encodeURIComponent(state.projectId) + "/envs/" + encodeURIComponent(env) + "/probe", {
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

    function previewApp() {
        if (!state.projectId) return;
        var previewWindow = window.open("about:blank", "_blank");
        if (!previewWindow) {
            var grid = document.getElementById("vibeDeployMain");
            if (grid) grid.innerHTML = '<div class="vibe-empty">Allow pop-ups to open the app preview.</div>';
            return;
        }
        previewWindow.document.body.innerHTML = "<p style='font-family:system-ui;padding:24px'>Resolving project preview…</p>";
        D.api("/api/vibe/projects/" + encodeURIComponent(state.projectId)).then(function (projectData) {
            var project = projectData && projectData.project;
            var env = (project && (project.environment || project.env)) || "development";
            return D.api("/api/vibe/projects/" + encodeURIComponent(state.projectId) + "/preview?env=" + encodeURIComponent(env));
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
        if (!state.projectId) return;
        var proj = state.projects.find(function (x) { return x.id === state.projectId; });
        var projName = proj ? proj.name : "project";
        var grid = document.getElementById("vibeDeployMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Launching deploy pipeline (approval-gated)...</div>';
        D.api("/api/vibe/run", {
            method: "POST",
            body: {
                intent: "Deploy " + projName,
                use_case: "software_development",
                pipeline_mode: "deploy",
                auto_approve: false,
                project_id: state.projectId,
                project_name: projName,
            },
        }).then(function (data) {
            if (data && data.success) {
                if (grid) {
                    grid.innerHTML = '<div class="vibe-empty">Deploy pipeline started for <b>' +
                        D.esc(projName) + "</b> — approve stages in the Run Dock.</div>";
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
