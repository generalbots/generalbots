/* Vibe Agents sidebar — split from vibe/vibe-agents.js for #1731 */

"use strict";

function deleteProject(p, force) {
    if (!p) return;
    var id = p.project_id || p.id;
    var name = p.name || "this project";
    if (!window.confirm("Delete project '" + name + "'?\n\nThis removes its VMs and workspace files. This cannot be undone.")) {
        return;
    }
    // Freeze the UI until the deletion finishes AND the project combo is
    // refreshed — the user must not interact mid-delete.
    var shell = window.VibeShell && window.VibeShell.toolbar;
    var setBusy = shell && typeof shell.setBusy === "function" ? shell.setBusy : null;
    var unfreeze = function () {
        if (setBusy) setBusy(false);
    };
    if (setBusy) setBusy(true, "Deleting project…");

    // Reload the toolbar's project combo and resolve when it is refreshed,
    // so the UI stays frozen until the dropdown reflects the deletion.
    var refreshCombo = function () {
        if (shell && typeof shell.loadProjects === "function") {
            var done = shell.loadProjects();
            if (done && typeof done.then === "function") return done;
        }
        return Promise.resolve();
    };

    vibeApi("/api/vibe/projects/" + encodeURIComponent(id) + (force ? "?force=true" : ""), {
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
                // #1288 — a deleted project must not linger in the restored
                // selection (localStorage) nor keep stale windows (canvas,
                // run dock, terminal, dialogs) showing its content.
                try {
                    localStorage.removeItem("gb-vibe-project-id");
                    sessionStorage.removeItem("gb-vibe-project-id");
                } catch (ignore) { }
                if (window.VibeWindows && typeof window.VibeWindows.closeVibeSubwindows === "function") {
                    window.VibeWindows.closeVibeSubwindows();
                }
                loadVibeProjects();
                document.dispatchEvent(new CustomEvent("gb:vibe-project", { detail: {} }));
                // Keep frozen until the combo is updated; translations and
                // the gb:vibe-project listener also reload, so the awaited
                // call here is the authoritative "combo ready" signal.
                return refreshCombo().then(unfreeze, unfreeze);
            } else {
                unfreeze();
                var msg = (data && data.error) || "delete failed";
                // #1440 — the default-bot protection answers with an explicit
                // explanation; offer a one-click force instead of a dead end.
                if (data && data.code === "default_bot_project_protected") {
                    if (window.confirm("Delete default bot project: " + msg)) {
                        deleteProject(p, true);
                    }
                    return;
                }
                window.alert("Could not delete project: " + msg);
            }
        })
        .catch(function (e) {
            unfreeze();
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
