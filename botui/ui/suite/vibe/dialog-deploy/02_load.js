/* Vibe Deploy dialog — split from vibe/vibe-dialog-deploy.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__deploy;
    var D = window.VibeDialogs;
    function loadProjects() {
        D.api("/api/vibe/projects").then(function (data) {
            M.state.projects = (data && data.projects) || [];
            var sel = document.getElementById("vibeDeployProject");
            if (!sel) return;
            sel.innerHTML = "";
            if (!M.state.projects.length) {
                sel.innerHTML = "<option>no projects yet</option>";
                return;
            }
            M.state.projects.forEach(function (p) {
                var opt = document.createElement("option");
                opt.value = p.id;
                opt.textContent = p.name || String(p.id).substring(0, 8);
                sel.appendChild(opt);
            });
            var wanted = M.state.projectId;
            if (wanted && document.querySelector('#vibeDeployProject option[value="' + wanted + '"]')) {
                sel.value = wanted;
            } else if (typeof currentProjectId !== "undefined" && currentProjectId) {
                sel.value = currentProjectId;
            }
            M.state.projectId = sel.value || null;
            var label = document.getElementById("vibeDeployLabel");
            if (label && M.state.projectId) {
                var p = M.state.projects.find(function (x) { return x.id === M.state.projectId; });
                label.textContent = p ? p.name : "project";
                label.className = "vibe-status ok";
            }
            if (M.state.projectId) M.loadHistory();
            if (M.state.projectId) M.loadSecurity();
        }).catch(function () {
            var sel = document.getElementById("vibeDeployProject");
            if (sel) sel.innerHTML = "<option>projects API unavailable</option>";
        });
    }

    function loadSecurity() {
        if (!M.state.projectId) return;
        var body = document.getElementById("vibeSecurityBody");
        if (!body) return;
        body.innerHTML = '<div class="vibe-empty">Loading domains...</div>';
        D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/domains").then(function (data) {
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
        if (!M.state.projectId) return;
        var list = document.getElementById("vibeDeployList");
        var grid = document.getElementById("vibeDeployMain");
        if (list) list.innerHTML = '<div class="vibe-empty">Loading deployments...</div>';
        D.api("/api/vibe/projects/" + encodeURIComponent(M.state.projectId) + "/deployments").then(function (data) {
            M.state.history = (data && (data.deployments || data.history)) || [];
            if (list) {
                if (!M.state.history.length) {
                    list.innerHTML = '<div class="vibe-empty">No deployments yet.</div>';
                } else {
                    list.innerHTML = "";
                    M.state.history.forEach(function (h, i) {
                        var row = D.el("div", "vibe-list-item");
                        var env = h.env || "—";
                        var st = h.status || h.state || "?";
                        var cls = String(st).toLowerCase() === "active" ? "ok" :
                            String(st).toLowerCase() === "failed" ? "err" : "info";
                        row.innerHTML = "<span>" + D.esc(env) + " #" + (M.state.history.length - i) + "</span>" +
                            '<span class="vibe-status ' + cls + '">' + D.esc(st) + "</span>";
                        list.appendChild(row);
                    });
                }
            }
            M.renderHistory();
        }).catch(function () {
            if (list) list.innerHTML = '<div class="vibe-empty">History unavailable (needs VM env).</div>';
            if (grid) grid.innerHTML = '<div class="vibe-empty">Deployment history requires Incus runtime (prod/staging env).</div>';
        });
    }

    function renderHistory() {
        var grid = document.getElementById("vibeDeployMain");
        if (!grid) return;
        if (!M.state.history.length) {
            grid.innerHTML = '<div class="vibe-empty">No deployment records. Launch the pipeline to create one.</div>';
            return;
        }
        var html = '<table class="vibe-table"><thead><tr><th>#</th><th>Env</th><th>Status</th><th>Run</th><th>When</th></tr></thead><tbody>';
        M.state.history.forEach(function (h, i) {
            var run = h.run_id || h.id;
            var runLabel = run ? "#" + String(run).substring(0, 4) : "—";
            var when = h.created_at || h.ts || "";
            if (when) when = String(when).replace("T", " ").substring(0, 16);
            html += "<tr><td>" + (M.state.history.length - i) + "</td>" +
                "<td>" + D.esc(h.env || "—") + "</td>" +
                "<td>" + D.esc(h.status || h.state || "?") + "</td>" +
                "<td>" + D.esc(runLabel) + "</td>" +
                "<td>" + D.esc(when || "—") + "</td></tr>";
        });
        html += "</tbody></table>";
        grid.innerHTML = html;
    }

    M.loadProjects = loadProjects;
    M.loadSecurity = loadSecurity;
    M.loadHistory = loadHistory;
    M.renderHistory = renderHistory;
})(window.VibeDialogs);
