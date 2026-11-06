/* Vibe Schema dialog — split from vibe/vibe-dialog-db.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__db = D.__db || {};
    var D = window.VibeDialogs;
    M.state = { tables: [], table: null, page: 1, pageSize: 100, cols: [], rows: [], pk: null, env: "production" };

    // Same resolution order as the Terminal pane: in-memory selection first,
    // then the persisted one so the dialog still works right after a reload.
    function projectId() {
        return (typeof window.currentProjectId !== "undefined" && window.currentProjectId) ||
            (function () {
                try { return sessionStorage.getItem("gb-vibe-project-id") || ""; } catch (e) { return ""; }
            })() || "";
    }

    // Requests tagged with the selected environment AND the selected project
    // so the backend connects to that project's matching database
    // (production default, `dev` = `_dev` twin).
    function apiEnv(path, options) {
        options = options || {};
        options.headers = options.headers || {};
        options.headers["X-Db-Env"] = M.state.env;
        var pid = M.projectId();
        if (pid) options.headers["X-Vibe-Project"] = pid;
        return D.api(path, options);
    }

    // Human-readable label of the database this dialog is pointed at.
    function dbName() {
        var pid = M.projectId();
        if (!pid) return "no project selected";
        var name = "";
        try { name = sessionStorage.getItem("gb-vibe-project-name") || ""; } catch (e) { }
        var slug = String(name).toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "").slice(0, 32) || "app";
        return "app_…_" + slug + (M.state.env === "dev" ? "_dev" : "");
    }

    M.projectId = projectId;
    M.apiEnv = apiEnv;
    M.dbName = dbName;
})(window.VibeDialogs);
