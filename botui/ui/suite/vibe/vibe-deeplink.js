/**
 * Vibe Deeplink consumer (#753) — inside the Vibe app partial.
 * Reads window.__gbAppParams__ (set by the shell's openDeepLink) and
 * location.search for ?vibe= / ?run_id=, then focuses the project
 * and/or loads a specific run into the surface.
 */
(function () {
    "use strict";

    function params() {
        var p = window.__gbAppParams__ || {};
        var qs = new URLSearchParams(window.location.search);
        if (!p.project && qs.get("project")) p.project = qs.get("project");
        if (!p.project && qs.get("vibe")) p.project = qs.get("vibe");
        if (!p.run_id && qs.get("run_id")) p.run_id = qs.get("run_id");
        return p;
    }

    function focusProject(projectId, notifyOnly) {
        if (!projectId) return;
        var name = String(projectId).toLowerCase();
        if (typeof currentProject !== "undefined") {
            if (!notifyOnly) currentProject = name;
        }
        var projectInput = document.getElementById("vibeProjectInput") ||
            document.querySelector("[data-gb-project]");
        if (projectInput) projectInput.value = name;
        document.dispatchEvent(new CustomEvent("gb:vibe-project", { detail: { project: name } }));
    }

    function focusRun(runId) {
        if (!runId) return;
        var id = String(runId);
        if (typeof vibeSessionId !== "undefined") vibeSessionId = id;
        document.dispatchEvent(new CustomEvent("gb:vibe-run", { detail: { run_id: id } }));
    }

    function apply() {
        var p = params();
        var focuses = p.project || p.run_id;
        if (!focuses) return;
        setTimeout(function () {
            if (p.project) focusProject(p.project, false);
            if (p.run_id) focusRun(p.run_id);
            var shell = document.getElementById("vibeWindow") || document.querySelector(".vibe-container");
            if (shell) shell.scrollIntoView({ behavior: "smooth", block: "start" });
        }, 250);
    }

    window.dispatchEvent(new Event("gb:vibe-deeplink-loaded"));
    window.VibeDeeplink = { apply: apply, focusProject: focusProject, focusRun: focusRun };
    apply();
})();