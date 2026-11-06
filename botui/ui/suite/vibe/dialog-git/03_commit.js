/* Vibe Source Control dialog — split from vibe/vibe-dialog-git.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__git;
    var D = window.VibeDialogs;
    function sidebar() {
        var box = D.el("div", "vibe-git-commit-panel");

        var head = D.el("div", "vibe-dialog-title", "CHANGES");
        var commitBox = D.el("div", "vibe-commit-box");
        var msg = D.el("textarea", "vibe-textarea");
        msg.id = "vibeGitMessage";
        msg.placeholder = "Commit message (Ctrl+Enter)";
        var commitBtn = D.el("button", "vibe-btn primary", "Commit");
        commitBtn.id = "vibeGitCommitBtn";
        commitBtn.addEventListener("click", M.commit);
        var commitPushBtn = D.el("button", "vibe-btn", "Commit & Push");
        commitPushBtn.id = "vibeGitCommitPushBtn";
        commitPushBtn.addEventListener("click", function () { M.commit(true); });
        commitBox.appendChild(msg);
        var btnRow = D.el("div", "vibe-git-btnrow");
        btnRow.appendChild(commitBtn);
        btnRow.appendChild(commitPushBtn);
        commitBox.appendChild(btnRow);

        var stageAll = D.el("button", "vibe-btn vibe-git-stageall", "Stage all");
        stageAll.addEventListener("click", function () {
            M.state.files.forEach(function (f) { M.state.staged[f.file] = true; });
            M.renderStagedList();
        });

        var list = D.el("div", "vibe-list vibe-git-changes");
        list.id = "vibeGitFileList";
        list.innerHTML = '<div class="vibe-empty">Loading status…</div>';

        box.appendChild(head);
        box.appendChild(commitBox);
        box.appendChild(stageAll);
        box.appendChild(list);
        return box;
    }

    function renderStagedList() {
        var list = document.getElementById("vibeGitFileList");
        if (!list) return;
        if (!M.state.files.length) {
            list.innerHTML = '<div class="vibe-empty">Working tree clean.</div>';
            return;
        }
        list.innerHTML = "";
        M.state.files.forEach(function (f) {
            var row = D.el("div", "vibe-list-item");
            var check = D.el("input", "vibe-git-check");
            check.type = "checkbox";
            check.checked = M.state.staged[f.file] !== false;
            check.addEventListener("change", function () {
                M.state.staged[f.file] = check.checked;
            });
            var label = D.el("span", "vibe-git-file", f.file);
            var badge = D.el("span", M.badgeClass(M.statusLetter(f)), M.statusLetter(f));
            row.appendChild(check);
            row.appendChild(label);
            row.appendChild(badge);
            row.addEventListener("click", function (ev) {
                if (ev.target === check) return;
                M.state.selected = f.file;
                M.renderCrumb();
                M.loadDiff();
            });
            list.appendChild(row);
        });
    }

    M.sidebar = sidebar;
    M.renderStagedList = renderStagedList;
})(window.VibeDialogs);
