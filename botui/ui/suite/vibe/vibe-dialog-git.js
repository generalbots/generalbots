/**
 * Vibe Source Control dialog — real git integration.
 * Sidebar: commit box + changed files (/api/git/status, /api/git/commit).
 * Main: file list, branches (/api/git/branches) and commit log
 * (/api/git/log), diff view (/api/git/diff/:file).
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { files: [], branch: null, selected: null, log: [] };

    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        var head = D.el("div", "vibe-dialog-title");
        head.textContent = "CHANGES";

        var commitBox = D.el("div", "vibe-commit-box");
        var msg = D.el("textarea", "vibe-textarea");
        msg.id = "vibeGitMessage";
        msg.placeholder = "Commit message (Ctrl+Enter)";
        var commitBtn = D.el("button", "vibe-btn primary", "Commit");
        commitBtn.id = "vibeGitCommitBtn";
        commitBtn.addEventListener("click", commit);
        var pushBtn = D.el("button", "vibe-btn", "Push");
        pushBtn.id = "vibeGitPushBtn";
        pushBtn.title = "Push the current branch";
        pushBtn.addEventListener("click", pushBranch);
        commitBox.appendChild(msg);
        commitBox.appendChild(commitBtn);
        commitBox.appendChild(pushBtn);

        var count = D.el("span", "vibe-status info", "0");
        count.id = "vibeGitCount";

        var list = D.el("div", "vibe-list");
        list.id = "vibeGitFileList";
        list.innerHTML = '<div class="vibe-empty">Loading status...</div>';

        var branchBar = D.el("div", "vibe-browser-status");
        branchBar.id = "vibeGitBranchBar";
        branchBar.innerHTML = '<span class="vibe-branch-label">branch: <b>main</b></span>';

        box.appendChild(head);
        box.appendChild(count);
        box.appendChild(commitBox);
        box.appendChild(list);
        box.appendChild(branchBar);
        return box;
    }

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";

        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var refresh = D.el("button", "vibe-btn", "↻ Refresh");
        refresh.addEventListener("click", function () { loadStatus(); loadLog(); });
        var diffBtn = D.el("button", "vibe-btn", "Diff");
        diffBtn.addEventListener("click", loadDiff);
        var spacer = D.el("span");
        spacer.style.flex = "1";
        toolbar.appendChild(refresh);
        toolbar.appendChild(diffBtn);
        toolbar.appendChild(spacer);

        var grid = D.el("div", "vibe-grid");
        grid.id = "vibeGitMain";
        grid.innerHTML = '<div class="vibe-empty">Load commit log or select a changed file.</div>';

        box.appendChild(toolbar);
        box.appendChild(grid);
        return box;
    }

    function statusClass(st) {
        st = String(st || "").toLowerCase();
        if (st === "modified" || st === "changed") return "warn";
        if (st === "untracked") return "info";
        if (st === "deleted") return "err";
        return "info";
    }

    function loadStatus() {
        var list = document.getElementById("vibeGitFileList");
        if (list) list.innerHTML = '<div class="vibe-empty">Loading status...</div>';
        D.api("/api/git/status?repo=" + encodeURIComponent(repoName())).then(function (data) {
            state.files = (data && data.files) || [];
            state.branch = (data && data.branch) || null;
            var count = document.getElementById("vibeGitCount");
            if (count) {
                count.textContent = state.files.length + " changed";
                count.className = "vibe-status " + (state.files.length ? "warn" : "ok");
            }
            if (!list) return;
            if (!state.files.length) {
                list.innerHTML = '<div class="vibe-empty">Working tree clean.</div>';
                return;
            }
            list.innerHTML = "";
            state.files.forEach(function (f) {
                var row = D.el("div", "vibe-list-item");
                if (state.selected === f.file) row.classList.add("active");
                row.innerHTML = "<span>" + D.esc(f.file) + "</span>" +
                    '<span class="vibe-status ' + statusClass(f.status) + '">' + D.esc(f.status) + "</span>";
                row.addEventListener("click", function () {
                    state.selected = f.file;
                    loadStatus();
                    loadDiff();
                });
                list.appendChild(row);
            });
        }).catch(function (err) {
            if (list) list.innerHTML = '<div class="vibe-empty">Status error: ' + D.esc(err) + "</div>";
        });
        loadBranches();
    }

    function loadBranches() {
        D.api("/api/git/branches?repo=" + encodeURIComponent(repoName())).then(function (data) {
            var branches = (data && data.branches) || [];
            var bar = document.getElementById("vibeGitBranchBar");
            if (!bar) return;
            // No branches (fresh repo): never show a blank dropdown — inform
            // like the vibe window does ("main" is the standard branch).
            if (!Array.isArray(branches) || !branches.length) {
                bar.innerHTML = '<span class="vibe-branch-label">branch: <b>main</b></span>';
                return;
            }
            var sel = document.createElement("select");
            sel.id = "vibeGitBranch";
            sel.className = "vibe-select";
            sel.style.flex = "1";
            branches.forEach(function (b) {
                var name = typeof b === "string" ? b : (b.name || "?");
                var opt = document.createElement("option");
                opt.value = name;
                opt.textContent = name + (name === state.branch ? " *" : "");
                sel.appendChild(opt);
            });
            bar.innerHTML = "<span>branch</span>";
            bar.appendChild(sel);
        }).catch(function () {
            var bar = document.getElementById("vibeGitBranchBar");
            if (bar) bar.innerHTML = '<span class="vibe-branch-label">branch: <b>main</b></span>';
        });
    }

    function loadLog() {
        D.api("/api/git/log?repo=" + encodeURIComponent(repoName())).then(function (data) {
            state.log = (data && data.log) || (data && data.commits) || [];
            renderLog();
        }).catch(function () { });
    }

    function renderLog() {
        var grid = document.getElementById("vibeGitMain");
        if (!grid) return;
        if (!state.log.length) {
            grid.innerHTML = '<div class="vibe-empty">No commits yet.</div>';
            return;
        }
        var html = '<table class="vibe-table"><thead><tr><th>Commit</th><th>Author</th><th>Message</th><th>When</th></tr></thead><tbody>';
        state.log.forEach(function (c) {
            var id = c.id || c.hash || "?";
            html += "<tr><td>" + D.esc(String(id).substring(0, 10)) + "</td>" +
                "<td>" + D.esc(c.author || "?") + "</td>" +
                "<td>" + D.esc(c.message || "") + "</td>" +
                "<td>" + D.esc(c.when || c.date || "") + "</td></tr>";
        });
        html += "</tbody></table>";
        grid.innerHTML = html;
    }

    function loadDiff() {
        var grid = document.getElementById("vibeGitMain");
        if (!state.selected) {
            renderLog();
            return;
        }
        if (grid) grid.innerHTML = '<div class="vibe-empty">Loading diff...</div>';
        D.api("/api/git/diff/" + encodeURIComponent(state.selected) + "?repo=" +
            encodeURIComponent(repoName())).then(function (data) {
            if (data && data.diff != null) {
                grid.innerHTML = '<div class="vibe-diff"><pre>' + D.esc(String(data.diff)) + "</pre></div>";
            } else if (data && data.error) {
                grid.innerHTML = '<div class="vibe-empty">' + D.esc(data.error) + "</div>";
            } else {
                renderLog();
            }
        }).catch(function () { renderLog(); });
    }

    function pushBranch() {
        var btn = document.getElementById("vibeGitPushBtn");
        if (btn) { btn.disabled = true; btn.textContent = "Pushing..."; }
        D.api("/api/git/push?repo=" + encodeURIComponent(repoName()), {
            method: "POST",
        }).then(function (data) {
            if (data && data.status === "failure") {
                alert("Push failed: " + ((data && data.error) || "unknown"));
            } else if (data && data.success === false) {
                alert("Push failed: " + ((data && data.error) || "unknown"));
            } else {
                loadStatus();
                loadLog();
            }
        }).catch(function (err) {
            alert("Push error: " + err);
        }).finally(function () {
            if (btn) { btn.disabled = false; btn.textContent = "Push"; }
        });
    }

    function commit() {
        var msg = document.getElementById("vibeGitMessage");
        var btn = document.getElementById("vibeGitCommitBtn");
        if (!msg || !msg.value.trim()) return;
        if (btn) { btn.disabled = true; btn.textContent = "Committing..."; }
        D.api("/api/git/commit", {
            method: "POST",
            body: { message: msg.value.trim(), repo: repoName() },
        }).then(function (data) {
            if (data && data.success) {
                msg.value = "";
                loadStatus();
                loadLog();
            } else {
                alert("Commit failed: " + ((data && data.error) || "unknown"));
            }
        }).catch(function (err) {
            alert("Commit error: " + err);
        }).finally(function () {
            if (btn) { btn.disabled = false; btn.textContent = "Commit"; }
        });
    }

    function repoName() {
        var name = "vibe";
        if (typeof currentProject !== "undefined" && currentProject) name = String(currentProject);
        return name;
    }

    document.addEventListener("keydown", function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
            var msg = document.getElementById("vibeGitMessage");
            if (msg && msg === document.activeElement) commit();
        }
    });

    D.register("git", {
        build: function (body) {
            body.appendChild(sidebar());
            body.appendChild(main());
            loadStatus();
            loadLog();
        },
        teardown: function () {
            state = { files: [], branch: null, selected: null, log: [] };
        },
    });
})();