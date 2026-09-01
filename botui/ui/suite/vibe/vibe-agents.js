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
        (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
        localStorage.getItem("token") ||
        localStorage.getItem("id_token") ||
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

function applyProjectSelection(p, persist) {
    if (!p) return;
    var id = p.project_id || p.id;
    var name = p.name || "Unnamed project";
    if (typeof currentProject !== "undefined") currentProject = name;
    if (typeof currentProjectId !== "undefined") currentProjectId = id;
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

/* ------------------------------------------------- project info + delete */

// Project information dialog: shows the registry record (kind, env, repo,
// status, timestamps) and offers Delete. Rendered as a floating tool window
// when the desktop shell is present, else a simple fixed panel.
var _projectInfoTarget = null;

// Formats a token count with thousands separators (e.g. 12,345).
function fmtTokens(n) {
    n = Number(n || 0);
    return n.toLocaleString ? n.toLocaleString("en-US") : String(n);
}

function projectInfoHtml(p) {
    var id = p.project_id || p.id;
    var name = p.name || "Unnamed project";
    var rows = [
        ["Name", name],
        ["ID", String(id || "")],
        ["Type", p.project_type || "—"],
        ["Environment", p.environment || "—"],
        ["Repository", p.repository || "—"],
        ["Framework", p.framework || "—"],
        ["Status", p.status || "—"],
        ["Created", p.created_at ? new Date(p.created_at).toLocaleString() : "—"],
        ["Updated", p.updated_at ? new Date(p.updated_at).toLocaleString() : "—"],
    ];
    return (
        '<div style="padding:14px 16px;font-size:12px;color:var(--text,#eee);min-width:360px;max-width:520px">' +
        '<div style="font-weight:800;font-size:13px;margin-bottom:10px;border-bottom:1px solid var(--border,#333);padding-bottom:8px">📁 ' +
        esc(name) +
        "</div>" +
        '<div id="vibe-pi-tokens" style="margin-bottom:4px">' +
        '<div style="display:flex;justify-content:space-between;gap:16px;padding:4px 0;border-bottom:1px solid var(--border,#222)">' +
        '<span style="color:var(--text-muted,#999)">Total tokens</span><b id="vibe-pi-total">…</b></div>' +
        '<div style="display:flex;justify-content:space-between;gap:16px;padding:4px 0;border-bottom:1px solid var(--border,#222)">' +
        '<span style="color:var(--text-muted,#999)">Input tokens</span><b id="vibe-pi-input">…</b></div>' +
        '<div style="display:flex;justify-content:space-between;gap:16px;padding:4px 0;border-bottom:1px solid var(--border,#222)">' +
        '<span style="color:var(--text-muted,#999)">Output tokens</span><b id="vibe-pi-output">…</b></div>' +
        "</div>" +
        rows
            .map(function (r) {
                return (
                    '<div style="display:flex;justify-content:space-between;gap:16px;padding:4px 0;border-bottom:1px solid var(--border,#222)">' +
                    '<span style="color:var(--text-muted,#999)">' +
                    esc(r[0]) +
                    "</span><b>" +
                    esc(r[1]) +
                    "</b></div>"
                );
            })
            .join("") +
        '<div id="vibe-pi-history" style="margin-top:10px">' +
        '<div style="font-weight:800;font-size:12px;margin-bottom:6px">🗂 Run history</div>' +
        '<div id="vibe-pi-runs" style="max-height:180px;overflow-y:auto;border:1px solid var(--border,#333);border-radius:6px">' +
        '<div class="vibe-empty" style="padding:10px">Loading history…</div></div></div>' +
        '<div style="display:flex;gap:8px;margin-top:14px;justify-content:flex-end">' +
        '<button type="button" class="vibe-btn" data-pi-close>Close</button>' +
        '<button type="button" class="vibe-btn" style="background:#ef4444;border-color:#ef4444;color:#fff" data-pi-delete="' +
        esc(id) +
        '">Delete project</button>' +
        "</div></div>"
    );
}

// Fetches the project run history (tokens + runs) and fills the Properties
// window placeholders when it is open.
function loadProjectInfoStats(p) {
    var id = (p && (p.project_id || p.id)) || "";
    if (!id || typeof vibeAuthFetch !== "function") return;
    vibeAuthFetch("/api/vibe/projects/" + encodeURIComponent(id) + "/history")
        .then(function (r) {
            if (!r.ok) throw new Error("history failed");
            return r.json();
        })
        .then(function (data) {
            function setText(elId, text) {
                var el = document.getElementById(elId);
                if (el) el.textContent = text;
            }
            var totals = (data && data.totals) || {};
            setText("vibe-pi-total", fmtTokens(totals.tokens));
            setText("vibe-pi-input", fmtTokens(totals.input_tokens));
            setText("vibe-pi-output", fmtTokens(totals.output_tokens));
            var box = document.getElementById("vibe-pi-runs");
            if (!box) return;
            var runs = (data && data.runs) || [];
            if (!runs.length) {
                box.innerHTML = '<div class="vibe-empty" style="padding:10px">No runs yet — start a Run or Chat with the agent.</div>';
                return;
            }
            box.innerHTML = runs.map(function (r) {
                var when = r.created_at ? new Date(r.created_at).toLocaleString() : "";
                var dur = "";
                if (r.created_at && r.completed_at) {
                    var ms = new Date(r.completed_at) - new Date(r.created_at);
                    if (ms >= 0) dur = " · " + Math.round(ms / 1000) + "s";
                }
                var tok = fmtTokens(r.tokens && r.tokens.tokens);
                var mode = r.pipeline_mode ? " [" + esc(r.pipeline_mode) + "]" : "";
                var err = r.error ? ' <span style="color:#f87171">' + esc(String(r.error).substring(0, 90)) + "</span>" : "";
                return (
                    '<div style="padding:6px 8px;border-bottom:1px solid var(--border,#222)">' +
                    '<div style="display:flex;justify-content:space-between;gap:10px"><b>' +
                    esc(String(r.state || "")) +
                    '</b><span style="color:var(--text-muted,#999);white-space:nowrap">' +
                    esc(tok) +
                    " tok" +
                    dur +
                    "</span></div>" +
                    '<div style="color:var(--text-muted,#bbb);word-break:break-word;margin-top:2px">' +
                    esc(String(r.intent || "")) +
                    mode +
                    err +
                    "</div></div>"
                );
            }).join("");
        })
        .catch(function () {
            var box = document.getElementById("vibe-pi-runs");
            if (box) box.innerHTML = '<div class="vibe-empty" style="padding:10px">History unavailable.</div>';
            ["vibe-pi-total", "vibe-pi-input", "vibe-pi-output"].forEach(function (elId) {
                var el = document.getElementById(elId);
                if (el) el.textContent = "—";
            });
        });
}

function showProjectInfo(p) {
    if (!p) {
        var fallbackId = typeof window.currentProjectId !== "undefined" ? window.currentProjectId : null;
        if (!fallbackId) return;
        p = { id: fallbackId, name: window.currentProject || String(fallbackId) };
    }
    _projectInfoTarget = p;
    if (window.VibeDialogs && window.VibeDialogs.open) {
        window.VibeDialogs.open("project", "Project Info");
        var toolBody = document.getElementById("window-body-vibe-tool-project");
        if (toolBody && !toolBody.textContent.trim()) {
            toolBody.innerHTML = projectInfoHtml(p);
            loadProjectInfoStats(p);
            var close = toolBody.querySelector("[data-pi-close]");
            if (close) close.addEventListener("click", function () {
                if (window.VibeDialogs) window.VibeDialogs.close();
            });
        }
        return;
    }
    // Fallback: simple floating panel.
    var float = document.createElement("div");
    float.style.cssText =
        "position:fixed;top:25%;left:35%;z-index:9999;background:var(--surface,#1a1a2e);" +
        "border:1px solid var(--border,#333);border-radius:10px;box-shadow:0 12px 40px rgba(0,0,0,.4);";
    float.innerHTML = projectInfoHtml(p);
    document.body.appendChild(float);
    float.querySelector("[data-pi-close]").addEventListener("click", function () { float.remove(); });
    var del = float.querySelector("[data-pi-delete]");
    if (del) del.addEventListener("click", function () {
        deleteProject(p);
        float.remove();
    });
}

// Register the project-info builder once so VibeDialogs.open("project") has
// a target; it renders whatever project was last clicked (or the selected one).
// vibe-agents.js loads before vibe-dialogs.js, so defer until the registry
// exists (DOMContentLoaded is late enough in the same partial).
function registerProjectInfoDialog() {
    if (!window.VibeDialogs || !window.VibeDialogs.register) return false;
    window.VibeDialogs.register("project", {
        build: function (body) {
            var p =
                _projectInfoTarget ||
                (typeof currentProjectId !== "undefined" && currentProjectId
                    ? { id: currentProjectId, name: currentProject }
                    : null);
            if (!p) {
                body.innerHTML = '<div class="vibe-empty">No project selected.</div>';
                return;
            }
            body.innerHTML = projectInfoHtml(p);
            loadProjectInfoStats(p);
            var close = body.querySelector("[data-pi-close]");
            if (close) close.addEventListener("click", function () {
                if (window.VibeDialogs) window.VibeDialogs.close();
            });
            var del = body.querySelector("[data-pi-delete]");
            if (del) del.addEventListener("click", function () {
                deleteProject(p);
                if (window.VibeDialogs) window.VibeDialogs.close();
            });
        },
    });
    return true;
}
// vibe-agents.js parses before vibe-dialogs.js in the same partial, and the
// partial is injected via HTMX into an already-complete document (where
// DOMContentLoaded never fires again), so a plain readyState check is not
// enough. Poll briefly until VibeDialogs exists with its fresh registry.
(function () {
    var tries = 0;
    function tryRegister() {
        if (registerProjectInfoDialog()) return;
        if (++tries < 50) setTimeout(tryRegister, 100);
    }
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", tryRegister);
    } else {
        tryRegister();
    }
})();

function deleteProject(p) {
    if (!p) return;
    var id = p.project_id || p.id;
    var name = p.name || "this project";
    if (!window.confirm("Delete project '" + name + "'?\n\nThis removes its VMs and workspace files. This cannot be undone.")) {
        return;
    }
    vibeApi("/api/vibe/projects/" + encodeURIComponent(id), {
        method: "DELETE",
    })
        .then(function (data) {
            if (data && data.success) {
                if (typeof vibeAddMsg === "function") {
                    vibeAddMsg("system", "🗑 Project '" + name + "' deleted.");
                }
                if (String(id) === String(currentProjectId || "")) {
                    currentProjectId = null;
                    currentProject = "";
                    window.currentProjectId = null;
                    window.currentProject = "";
                }
                loadVibeProjects();
                document.dispatchEvent(new CustomEvent("gb:vibe-project", { detail: {} }));
            } else {
                var msg = (data && data.error) || "delete failed";
                window.alert("Could not delete project: " + msg);
            }
        })
        .catch(function (e) {
            window.alert("Could not delete project: " + e.message);
        });
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
