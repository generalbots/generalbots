/* Vibe Schema dialog — split from vibe/vibe-dialog-db.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__db;
    var D = window.VibeDialogs;
    D.register("db", {
        build: function (body) {
            body.appendChild(M.sidebar());
            body.appendChild(M.main());
            M.refreshCtxChip();
            M.loadSchema(false);
            var prev = document.getElementById("vibeDbPrev");
            var next = document.getElementById("vibeDbNext");
            if (prev) prev.addEventListener("click", function () {
                if (M.state.page > 1) { M.state.page--; M.loadTableData(); }
            });
            if (next) next.addEventListener("click", function () {
                M.state.page++; M.loadTableData();
            });
        },
        teardown: function () {
            M.state = { tables: [], table: null, page: 1, pageSize: 100, cols: [], rows: [], pk: null, env: "production" };
        },
    });
    // #1408 — a finished Vibe agent run may have changed the schema/data;
    // reload it when the Database pane is open.
    document.addEventListener('gb:vibe-app-refresh', function () {
        M.loadSchema(false);
    });
})(window.VibeDialogs);
