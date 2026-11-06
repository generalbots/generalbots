/* Vibe Deploy dialog — split from vibe/vibe-dialog-deploy.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__deploy;
    var D = window.VibeDialogs;
    D.register("deploy", {
        build: function (body) {
            body.appendChild(M.sidebar());
            body.appendChild(M.main());
            M.loadProjects();
        },
        teardown: function () {
            M.state = { projects: [], projectId: null, history: [] };
        },
    });
})(window.VibeDialogs);
