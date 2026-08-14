/**
 * Vibe sidebar — real data (2026-08-14 cleanup).
 *
 * The previous implementation simulated a "vibe farm": hardcoded Vibe #1-#4
 * agent cards (EVOLVED/BRED/WILD badges), DOM-only workspace items and
 * drag-and-drop assignment. None of it had a backend, so it was removed.
 *
 * The sidebar now lists real projects from /api/vibe/projects. Selecting a
 * project dispatches `gb:vibe-project` (Members + dialogs react), and
 * "+ Create a New Project" opens the real New Project modal.
 */
"use strict";

function authHeaders(extra) {
    var headers = Object.assign({}, extra || {});
    var token =
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        "";
    if (token) headers["Authorization"] = "Bearer " + token;
    return headers;
}

function vibeApi(path, options) {
    options = options || {};
    options.headers = authHeaders(options.headers || {});
    if (options.body && typeof options.body !== "string") {
        options.headers["Content-Type"] = "application/json";
        options.body = JSON.stringify(options.body);
    }
    return fetch(path, options).then(function (r) {
        return r.json().catch(function () {
            return { success: false, error: "HTTP " + r.status };
        });
    });
}

function esc(s) {
    var d = document.createElement("div");
    d.textContent = s == null ? "" : String(s);
    return d.innerHTML;
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
                    '<button class="as-workspace-toggle" type="button" style="background: var(--bg);' +
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
                    "</button>";
                item
                    .querySelector(".as-workspace-toggle")
                    .addEventListener("click", function () {
                        selectProject(p);
                    });
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
    var id = p.project_id || p.id;
    var name = p.name || "Unnamed project";
    if (typeof currentProject !== "undefined") currentProject = name;
    if (typeof currentProjectId !== "undefined") currentProjectId = id;
    var trail = document.querySelector(".vibe-trail");
    if (trail) trail.textContent = "// " + String(name).toUpperCase();
    document.dispatchEvent(
        new CustomEvent("gb:vibe-project", { detail: { id: id, project: name } }),
    );
    loadVibeProjects();
}

/**
 * Legacy hooks kept as safe no-ops — the fake agent cards were removed
 * (2026-08-14). Callers (vibe-run.js onProgress, vibe-websocket.js) still
 * invoke them; progress now lives in the Run Dock instead.
 */
function updateVibe1(status, detail) {
    void status;
    void detail;
}

function updateAgentCard(agentId, status, detail) {
    void agentId;
    void status;
    void detail;
}

document.addEventListener("gb:vibe-project-created", function () {
    loadVibeProjects();
});
document.addEventListener("gb:vibe-project", function () {
    loadVibeProjects();
});

(function () {
    var __cb = function () {
        loadVibeProjects();
    };
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", __cb);
    } else {
        __cb();
    }
})();
