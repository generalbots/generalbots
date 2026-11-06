/* Vibe Schema dialog — split from vibe/vibe-dialog-db.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__db;
    var D = window.VibeDialogs;
    // #1386b — keep the context chip in sync with the selection and env.
    function refreshCtxChip() {
        var chip = document.getElementById("vibeDbCtx");
        if (!chip) return;
        var pid = M.projectId();
        if (!pid) {
            chip.textContent = "⛔ no project selected";
            chip.className = "vibe-status err";
            return;
        }
        chip.textContent = "🗄️ " + M.dbName();
        chip.className = "vibe-status ok";
    }

    // A project selected in ANY Vibe window (main or specialist) retargets
    // this dialog and reloads the schema — same broadcast the Terminal uses.
    document.addEventListener("gb:vibe-project", function () {
        M.refreshCtxChip();
        M.state.table = null;
        M.state.page = 1;
        M.loadSchema(false);
    });

    M.refreshCtxChip = refreshCtxChip;
})(window.VibeDialogs);
