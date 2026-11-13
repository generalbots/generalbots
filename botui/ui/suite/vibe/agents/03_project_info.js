/* Vibe Agents sidebar — split from vibe/vibe-agents.js for #1731 */

"use strict";

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
        ["Created", p.created_at ? gbFormatDateTime(new Date(p.created_at)) : "—"],
        ["Updated", p.updated_at ? gbFormatDateTime(new Date(p.updated_at)) : "—"],
    ];
    return (
        '<div style="padding:14px 16px;font-size:12px;color:var(--text,#eee);min-width:360px;max-width:520px">' +
        '<div style="font-weight:800;font-size:13px;margin-bottom:10px;border-bottom:1px solid var(--border,#333);padding-bottom:8px"><span style="display:inline-flex;vertical-align:-2px;margin-right:5px" aria-hidden="true"><svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg></span>' +
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
                var when = r.created_at ? gbFormatDateTime(new Date(r.created_at)) : "";
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

// Render the project-info body into a host element (desktop tool window or
// floating panel) and wire its actions. Does NOT depend on the VibeDialogs
// registry: Properties must open even when scripts race and the 'project'
// builder was never registered.
function renderProjectInfoHost(host, p, closeFn) {
    if (!host) return false;
    host.innerHTML = projectInfoHtml(p);
    loadProjectInfoStats(p);
    var close = host.querySelector("[data-pi-close]");
    if (close) close.addEventListener("click", closeFn || noop);
    var del = host.querySelector("[data-pi-delete]");
    if (del) del.addEventListener("click", function () {
        deleteProject(p);
        (closeFn || noop)();
    });
    return true;
}

function noop() {}

// The target project for Properties: the clicked one, else the selected one;
// never a bare id — carry the project_type so the dialog and gating agree.
function projectInfoTarget(p) {
    if (p) return p;
    var id =
        (typeof window.currentProjectId !== "undefined" && window.currentProjectId) ||
        (function () {
            try {
                return (
                    sessionStorage.getItem("gb-vibe-project-id") ||
                    localStorage.getItem("gb-vibe-project-id") ||
                    null
                );
            } catch (e) {
                return null;
            }
        })();
    if (!id) return null;
    return {
        id: id,
        name: window.currentProject || String(id),
        project_type: window.currentProjectKind || "",
    };
}

function showProjectInfo(p) {
    var target = projectInfoTarget(p);
    if (!target) return;
    _projectInfoTarget = target;
    // Prefer a real desktop tool window on the WindowManager (works whether
    // or not VibeDialogs registered the 'project' builder). Pinned to the
    // top of the workspace ("lá em cima") instead of below the Vibe bar.
    var wm = window.WindowManager;
    if (wm && typeof wm.openToolWindowBody === "function") {
        var wmBody = wm.openToolWindowBody("vibe-tool-project", "Project Info", {
            ownerId: "vibe",
            top: 16,
        });
        if (wmBody) {
            wmBody.innerHTML = "";
            var wrap = document.createElement("div");
            wrap.className = "vibe-dialog vibe-dialog-toolwindow";
            wmBody.appendChild(wrap);
            var closeWm = function () {
                if (wm && typeof wm.close === "function") wm.close("vibe-tool-project");
            };
            renderProjectInfoHost(wrap, target, closeWm);
            if (typeof wm.focusWindow === "function") wm.focusWindow("vibe-tool-project");
            return;
        }
    }
    // Isolated fallback: floating panel appended to the document body,
    // pinned near the top as well.
    var float = document.createElement("div");
    float.className = "vibe-dialog vibe-dialog-float vibe-dialog-host-project";
    float.style.cssText =
        "position:fixed;top:16px;left:50%;transform:translateX(-50%);z-index:9999;background:var(--gb-surface,#1a1a2e);" +
        "border:1px solid var(--gb-border,#333);border-radius:10px;box-shadow:0 12px 40px rgba(0,0,0,.4);";
    renderProjectInfoHost(float, target, function () { float.remove(); });
    document.body.appendChild(float);
}

// Register the project-info builder once so VibeDialogs.open("project") also
// has a target; it renders whatever project was last clicked (or the selected
// one). vibe-agents.js loads before vibe-dialogs.js, so defer until the
// registry exists (DOMContentLoaded is late enough in the same partial).
function registerProjectInfoDialog() {
    if (!window.VibeDialogs || !window.VibeDialogs.register) return false;
    window.VibeDialogs.register("project", {
        build: function (body, apiCtx) {
            var p =
                _projectInfoTarget ||
                projectInfoTarget(null);
            if (!p) {
                body.innerHTML = '<div class="vibe-empty">No project selected.</div>';
                return;
            }
            renderProjectInfoHost(body, p, apiCtx && typeof apiCtx.close === "function" ? apiCtx.close : null);
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
