/* Vibe Source Control dialog — split from vibe/vibe-dialog-git.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__git;
    var D = window.VibeDialogs;
    function refreshAll() {
        M.loadStatus();
        M.loadLog();
        if (M.state.selected) M.loadDiff(); else M.loadTree();
    }

    function loadStatus() {
        D.api("/api/git/status?repo=" + encodeURIComponent(M.repoName()))
            .then(function (data) {
                M.state.files = (data && data.files) || [];
                M.state.branch = (data && data.branch) || null;
                var count = document.getElementById("vibeGitCount");
                if (count) {
                    count.textContent = M.state.files.length + " changed";
                    count.className = "vibe-status " + (M.state.files.length ? "warn" : "ok");
                }
                M.renderStagedList();
                if (!M.state.selected) M.loadTree();
                M.loadBranches();
            })
            .catch(function (err) {
                var list = document.getElementById("vibeGitFileList");
                if (list) list.innerHTML = '<div class="vibe-empty">Status error: ' + D.esc(err) + "</div>";
            });
    }

    function loadBranches() {
        D.api("/api/git/branches?repo=" + encodeURIComponent(M.repoName()))
            .then(function (data) {
                var branches = (data && data.branches) || [];
                var sel = document.getElementById("vibeGitBranch");
                if (!sel) return;
                sel.innerHTML = "";
                var names = (Array.isArray(branches) && branches.length)
                    ? branches.map(function (b) { return typeof b === "string" ? b : (b.name || "?"); })
                    : ["main"];
                names.forEach(function (name) {
                    var opt = document.createElement("option");
                    opt.value = name;
                    opt.textContent = name + (name === M.state.branch ? " *" : "");
                    sel.appendChild(opt);
                });
                if (M.state.branch && names.indexOf(M.state.branch) !== -1) sel.value = M.state.branch;
            })
            .catch(function () { /* branch bar keeps its previous content */ });
    }

    function switchBranch(name) {
        if (!name) return;
        D.api("/api/git/branch/" + encodeURIComponent(name) + "?repo=" + encodeURIComponent(M.repoName()), {
            method: "POST",
        }).then(function (data) {
            if (data && data.status === "failure") {
                alert("Branch switch failed: " + ((data && data.error) || "unknown"));
            }
            M.state.selected = null;
            M.refreshAll();
        }).catch(function (err) { alert("Branch error: " + err); });
    }

    function loadLog() {
        D.api("/api/git/log?repo=" + encodeURIComponent(M.repoName()))
            .then(function (data) {
                M.state.log = (data && data.log) || (data && data.commits) || [];
            })
            .catch(function () { });
    }

    function pullBranch() {
        D.api("/api/git/pull?repo=" + encodeURIComponent(M.repoName()), { method: "POST" })
            .then(function (data) {
                if (data && data.status === "failure") {
                    alert("Pull failed: " + ((data && data.error) || "unknown"));
                } else {
                    M.refreshAll();
                }
            })
            .catch(function (err) { alert("Pull error: " + err); });
    }

    function pushBranch() {
        var btn = document.getElementById("vibeGitPushBtn");
        if (btn) { btn.disabled = true; btn.textContent = "Pushing…"; }
        D.api("/api/git/push?repo=" + encodeURIComponent(M.repoName()), { method: "POST" })
            .then(function (data) {
                if (data && (data.status === "failure" || data.success === false)) {
                    alert("Push failed: " + ((data && data.error) || "unknown"));
                } else {
                    M.loadStatus();
                    M.loadLog();
                }
            })
            .catch(function (err) { alert("Push error: " + err); })
            .finally(function () {
                if (btn) { btn.disabled = false; btn.textContent = "⤒ Push"; }
            });
    }

    function commit(andPush) {
        var msg = document.getElementById("vibeGitMessage");
        var btn = document.getElementById("vibeGitCommitBtn");
        if (!msg || !msg.value.trim()) return;
        if (btn) { btn.disabled = true; btn.textContent = "Committing…"; }
        var files = M.state.files
            .filter(function (f) { return M.state.staged[f.file] !== false; })
            .map(function (f) { return f.file; });
        D.api("/api/git/commit", {
            method: "POST",
            body: { message: msg.value.trim(), repo: M.repoName(), files: files },
        }).then(function (data) {
            if (data && (data.success === true || data.status === "success")) {
                msg.value = "";
                M.state.staged = {};
                if (andPush) M.pushBranch();
                M.refreshAll();
            } else {
                alert("Commit failed: " + ((data && data.error) || "unknown"));
            }
        }).catch(function (err) {
            alert("Commit error: " + err);
        }).finally(function () {
            if (btn) { btn.disabled = false; btn.textContent = "Commit"; }
        });
    }

    M.refreshAll = refreshAll;
    M.loadStatus = loadStatus;
    M.loadBranches = loadBranches;
    M.switchBranch = switchBranch;
    M.loadLog = loadLog;
    M.pullBranch = pullBranch;
    M.pushBranch = pushBranch;
    M.commit = commit;
})(window.VibeDialogs);
