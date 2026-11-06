/* Vibe Agents sidebar — split from vibe/vibe-agents.js for #1731 */

"use strict";

function applyProjectSelection(p, persist) {
    if (!p) return;
    var id = p.project_id || p.id;
    var name = p.name || "Unnamed project";
    if (typeof currentProject !== "undefined") currentProject = name;
    if (typeof currentProjectId !== "undefined") currentProjectId = id;
    // #1386 follow-up — the toolbar Chat button needs the project kind to
    // decide between a { bot: slug } deep-link (bot projects run as their
    // own bot) and the @project mention (website/apps agent contexts).
    window.currentProjectKind = String(p.project_type || "").toLowerCase();
    // Keep the explicit window properties synchronized for shell modules that
    // read state through window.VibeShell.
    window.currentProject = name;
    window.currentProjectId = id;
    if (persist !== false) {
        try {
            // Persist to BOTH storage backends: localStorage so the last-used
            // project is restored on a fresh tab/session (otherwise the shell
            // falls back to projects[0], which may have no web app yet and
            // shows "No web app in this project yet"), and sessionStorage for
            // the in-window sync used by loadProjects today.
            localStorage.setItem("gb-vibe-project-id", String(id || ""));
            localStorage.setItem("gb-vibe-project-name", String(name));
            localStorage.setItem("gb-vibe-project-kind", String(p.project_type || ""));
            sessionStorage.setItem("gb-vibe-project-id", String(id || ""));
            sessionStorage.setItem("gb-vibe-project-name", String(name));
        } catch (_) {
            // Storage can be disabled; the in-memory selection remains valid.
        }
    }
    var trail = document.querySelector(".vibe-trail");
    if (trail) trail.textContent = "// " + String(name).toUpperCase();
    document.dispatchEvent(
        new CustomEvent("gb:vibe-project", { detail: { id: id, project: name } }),
    );
}

function loadVibeProjects() {
    var list = document.getElementById("asProjectList");
    if (!list) return;
    list.innerHTML =
        '<div class="vibe-rd-empty" style="padding: 8px 12px; font-size: 12px;">Loading projects…</div>';
    vibeApi("/api/vibe/projects")
        .then(function (data) {
            var projects =
                (data && data.success && data.projects) ||
                (data && data.projects) ||
                [];
            if (!projects.length) {
                list.innerHTML =
                    '<div class="vibe-rd-empty" style="padding: 8px 12px; font-size: 12px;">' +
                    "No projects yet — create one to start building.</div>";
                return;
            }
            if (typeof currentProjectId !== "undefined" && !currentProjectId) {
                var storedId = "";
                try {
                    storedId = sessionStorage.getItem("gb-vibe-project-id") || "";
                } catch (_) {
                    storedId = "";
                }
                var preferred = projects.find(function (p) {
                    var id = p.project_id || p.id;
                    return storedId && String(id) === String(storedId);
                });
                if (!preferred) {
                    preferred = projects.find(function (p) {
                        return String(p.status || "").toLowerCase() === "active";
                    });
                }
                applyProjectSelection(preferred || projects[0], false);
            }
            list.innerHTML = "";
            projects.forEach(function (p) {
                var id = p.project_id || p.id;
                var name = p.name || p.project_type || "Unnamed project";
                var status = p.status || "";
                var active =
                    typeof currentProjectId !== "undefined" &&
                    currentProjectId &&
                    id &&
                    String(id) === String(currentProjectId);
                var item = document.createElement("div");
                item.className = "as-workspace-item";
                item.innerHTML =
                    '<button class="as-workspace-toggle" type="button" style="background:' + (active ? "var(--bg);" : "var(--bg);") +
                    (active ? "border-left: 3px solid var(--accent);" : "") +
                    '">' +
                    '<span class="as-workspace-arrow">▶</span>' +
                    '<span style="flex:1;text-align:left;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">' +
                    esc(name) +
                    "</span>" +
                    (status
                        ? '<span class="vibe-chip ' +
                          (status === "active" ? "state-completed" : "state-pending") +
                          '">' +
                          esc(status) +
                          "</span>"
                        : "") +
                    "</button>" +
                    '<span class="as-workspace-actions">' +
                    '<button type="button" class="as-ws-info" title="Project info" data-info="' + esc(id) + '">ⓘ</button>' +
                    '<button type="button" class="as-ws-delete" title="Delete project" data-delete="' + esc(id) + '">🗑</button>' +
                    "</span>";
                item
                    .querySelector(".as-workspace-toggle")
                    .addEventListener("click", function () {
                        selectProject(p);
                    });
                var infoBtn = item.querySelector(".as-ws-info");
                if (infoBtn) {
                    infoBtn.addEventListener("click", function (ev) {
                        ev.stopPropagation();
                        showProjectInfo(p);
                    });
                }
                var delBtn = item.querySelector(".as-ws-delete");
                if (delBtn) {
                    delBtn.addEventListener("click", function (ev) {
                        ev.stopPropagation();
                        deleteProject(p);
                    });
                }
                list.appendChild(item);
            });
        })
        .catch(function () {
            list.innerHTML =
                '<div class="vibe-rd-empty" style="padding: 8px 12px; font-size: 12px;">' +
                "Could not load projects.</div>";
        });
}

function selectProject(p) {
    applyProjectSelection(p, true);
    // User-initiated project switch: close every floating accessory so
    // project-scoped content (canvas, graph, metrics, run dock, dialogs)
    // from the previous project never lingers.
    if (window.VibeWindows && typeof window.VibeWindows.closeVibeSubwindows === "function") {
        window.VibeWindows.closeVibeSubwindows();
    }
    loadVibeProjects();
}
