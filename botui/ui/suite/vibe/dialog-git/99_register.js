/* Vibe Source Control dialog — split from vibe/vibe-dialog-git.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__git;
    var D = window.VibeDialogs;
    document.addEventListener("keydown", function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
            var msg = document.getElementById("vibeGitMessage");
            if (msg && msg === document.activeElement) M.commit(false);
        }
    });

    D.register("git", {
        build: function (body) {
            body.classList.add("vibe-git-dialog");
            body.appendChild(M.toolbar());
            var shell = D.el("div", "vibe-git-shell");
            shell.appendChild(M.center());
            shell.appendChild(M.sidebar());
            body.appendChild(shell);
            M.loadStatus();
            M.loadLog();
            M.loadTree();
        },
        teardown: function () {
            M.state = { files: [], tree: [], branch: null, path: "", selected: null, log: [], staged: {} };
        },
    });
})(window.VibeDialogs);
