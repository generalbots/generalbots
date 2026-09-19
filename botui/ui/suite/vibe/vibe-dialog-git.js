/**
 * Vibe Source Control dialog — GitHub-style review view (#1506).
 *
 * Layout:
 *   Toolbar : branch selector · Fetch · Pull · Push · ±N chip · Commit CTA
 *   Center  : repository tree (folders + files, GitHub "Files" view) with
 *             M/U/A/D/R diff badges on changed rows; clicking a changed file
 *             opens its diff with a breadcrumb back to the tree.
 *   Right   : manual commit panel — message (Ctrl+Enter), per-file staging
 *             checkboxes, Commit / Commit & Push.
 *
 * APIs: /api/git/status, /api/git/tree, /api/git/branches, /api/git/branch/:name,
 * /api/git/log, /api/git/diff/:file, /api/git/commit (files[] pathspec),
 * /api/git/pull, /api/git/push.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = {
        files: [],          // changed files from /api/git/status
        tree: [],           // current tree entries (center pane)
        branch: null,
        path: "",           // current tree directory ("")
        selected: null,     // file whose diff is open (null = tree view)
        log: [],
        staged: {},         // path -> bool
    };

    /* ── Status badges (GitHub letter + color) ──────────────────────────── */
    function badgeClass(letter) {
        switch (letter) {
            case "M": return "vibe-git-badge warn";
            case "U":
            case "A": return "vibe-git-badge ok";
            case "D": return "vibe-git-badge err";
            case "R": return "vibe-git-badge info";
            default: return "vibe-git-badge";
        }
    }

    function statusLetter(f) {
        var st = String(f.status || "").toLowerCase();
        if (st === "modified" || st === "changed") return "M";
        if (st === "untracked") return "U";
        if (st === "added") return "A";
        if (st === "deleted") return "D";
        if (st === "renamed") return "R";
        return "?";
    }

    function statusMap() {
        var map = {};
        state.files.forEach(function (f) { map[f.file] = statusLetter(f); });
        return map;
    }

    function repoName() {
        var name = "vibe";
        if (typeof currentProject !== "undefined" && currentProject) name = String(currentProject);
        return name;
    }

    /* ── Toolbar ────────────────────────────────────────────────────────── */
    function toolbar() {
        var bar = D.el("div", "vibe-git-toolbar");

        var branchSel = D.el("select", "vibe-select vibe-git-branch-sel");
        branchSel.id = "vibeGitBranch";
        branchSel.title = "Current branch";
        branchSel.addEventListener("change", function () {
            switchBranch(branchSel.value);
        });

        var fetchBtn = D.el("button", "vibe-btn", "↻ Fetch");
        fetchBtn.addEventListener("click", function () { refreshAll(); });

        var pullBtn = D.el("button", "vibe-btn", "⤓ Pull");
        pullBtn.addEventListener("click", pullBranch);

        var pushBtn = D.el("button", "vibe-btn", "⤒ Push");
        pushBtn.id = "vibeGitPushBtn";
        pushBtn.addEventListener("click", pushBranch);

        var count = D.el("span", "vibe-status warn", "0");
        count.id = "vibeGitCount";
        count.title = "Changed files";

        var spacer = D.el("span", "vibe-git-spacer");

        bar.appendChild(branchSel);
        bar.appendChild(fetchBtn);
        bar.appendChild(pullBtn);
        bar.appendChild(pushBtn);
        bar.appendChild(spacer);
        bar.appendChild(count);
        return bar;
    }

    /* ── Center: tree / diff ────────────────────────────────────────────── */
    function center() {
        var box = D.el("div", "vibe-git-center");
        var crumb = D.el("div", "vibe-git-breadcrumb");
        crumb.id = "vibeGitCrumb";
        var grid = D.el("div", "vibe-git-tree");
        grid.id = "vibeGitMain";
        grid.innerHTML = '<div class="vibe-empty">Loading repository…</div>';
        box.appendChild(crumb);
        box.appendChild(grid);
        return box;
    }

    function renderCrumb() {
        var crumb = document.getElementById("vibeGitCrumb");
        if (!crumb) return;
        crumb.innerHTML = "";
        if (!state.selected) {
            var repoLink = D.el("span", "vibe-git-crumb-link", repoName());
            repoLink.addEventListener("click", function () { openTree(""); });
            crumb.appendChild(repoLink);
            if (state.path) {
                var parts = state.path.split("/");
                var acc = "";
                parts.forEach(function (part) {
                    acc = acc ? acc + "/" + part : part;
                    crumb.appendChild(D.el("span", "vibe-git-crumb-sep", " / "));
                    var seg = acc;
                    var link = D.el("span", "vibe-git-crumb-link", part);
                    link.addEventListener("click", function () { openTree(seg); });
                    crumb.appendChild(link);
                });
            }
        } else {
            var back = D.el("span", "vibe-git-crumb-link", "← " + repoName());
            back.addEventListener("click", function () {
                state.selected = null;
                openTree(state.path);
            });
            crumb.appendChild(back);
            crumb.appendChild(D.el("span", "vibe-git-crumb-sep", " / "));
            crumb.appendChild(D.el("span", "vibe-git-crumb-file", state.selected));
        }
    }

    function openTree(path) {
        state.path = path || "";
        state.selected = null;
        renderCrumb();
        loadTree();
    }

    function loadTree() {
        var grid = document.getElementById("vibeGitMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Loading tree…</div>';
        D.api("/api/git/tree?repo=" + encodeURIComponent(repoName()) +
            "&path=" + encodeURIComponent(state.path))
            .then(function (data) {
                state.tree = (data && data.entries) || [];
                renderTree();
            })
            .catch(function (err) {
                if (grid) grid.innerHTML = '<div class="vibe-empty">Tree error: ' + D.esc(err) + "</div>";
            });
    }

    function renderTree() {
        var grid = document.getElementById("vibeGitMain");
        if (!grid) return;
        var map = statusMap();
        if (!state.tree.length) {
            grid.innerHTML = '<div class="vibe-empty">Empty directory.</div>';
            return;
        }
        var list = D.el("div", "vibe-git-list");
        state.tree.forEach(function (entry) {
            var row = D.el("div", "vibe-git-row" + (entry.is_dir ? " is-dir" : ""));
            var icon = D.el("span", "vibe-git-icon", entry.is_dir ? "📁" : "📄");
            var name = D.el("span", "vibe-git-name", fileName(entry.path));
            row.appendChild(icon);
            row.appendChild(name);
            if (map[entry.path]) {
                row.appendChild(D.el("span", badgeClass(map[entry.path]), map[entry.path]));
            }
            row.addEventListener("click", function () {
                if (entry.is_dir) {
                    openTree(entry.path);
                } else {
                    state.selected = entry.path;
                    renderCrumb();
                    loadDiff();
                }
            });
            list.appendChild(row);
        });
        grid.innerHTML = "";
        grid.appendChild(list);
    }

    function fileName(path) {
        var parts = String(path || "").split("/");
        return parts[parts.length - 1] || path;
    }

    function loadDiff() {
        var grid = document.getElementById("vibeGitMain");
        if (!state.selected) {
            loadTree();
            return;
        }
        renderCrumb();
        if (grid) grid.innerHTML = '<div class="vibe-empty">Loading diff…</div>';
        D.api("/api/git/diff/" + encodeURIComponent(state.selected) + "?repo=" +
            encodeURIComponent(repoName()))
            .then(function (data) {
                if (data && data.diff != null && String(data.diff).trim()) {
                    grid.innerHTML = '<div class="vibe-diff"><pre>' +
                        highlightDiff(String(data.diff)) + "</pre></div>";
                } else if (data && data.error) {
                    grid.innerHTML = '<div class="vibe-empty">' + D.esc(data.error) + "</div>";
                } else {
                    grid.innerHTML = '<div class="vibe-empty">No textual changes (binary or untracked file).</div>';
                }
            })
            .catch(function () { loadTree(); });
    }

    /* Diff line highlighting: +green / -red / @@hunks (style only). */
    function highlightDiff(text) {
        return text.split("\n").map(function (line) {
            var esc = D.esc(line);
            if (line.indexOf("+++") === 0 || line.indexOf("---") === 0 || line.indexOf("diff ") === 0) {
                return '<span class="vibe-diff-meta">' + esc + "</span>";
            }
            if (line.indexOf("@@") === 0) return '<span class="vibe-diff-hunk">' + esc + "</span>";
            if (line.indexOf("+") === 0) return '<span class="vibe-diff-add">' + esc + "</span>";
            if (line.indexOf("-") === 0) return '<span class="vibe-diff-del">' + esc + "</span>";
            return esc;
        }).join("\n");
    }

    /* ── Right: manual commit panel ────────────────────────────────────── */
    function sidebar() {
        var box = D.el("div", "vibe-git-commit-panel");

        var head = D.el("div", "vibe-dialog-title", "CHANGES");
        var commitBox = D.el("div", "vibe-commit-box");
        var msg = D.el("textarea", "vibe-textarea");
        msg.id = "vibeGitMessage";
        msg.placeholder = "Commit message (Ctrl+Enter)";
        var commitBtn = D.el("button", "vibe-btn primary", "Commit");
        commitBtn.id = "vibeGitCommitBtn";
        commitBtn.addEventListener("click", commit);
        var commitPushBtn = D.el("button", "vibe-btn", "Commit & Push");
        commitPushBtn.id = "vibeGitCommitPushBtn";
        commitPushBtn.addEventListener("click", function () { commit(true); });
        commitBox.appendChild(msg);
        var btnRow = D.el("div", "vibe-git-btnrow");
        btnRow.appendChild(commitBtn);
        btnRow.appendChild(commitPushBtn);
        commitBox.appendChild(btnRow);

        var stageAll = D.el("button", "vibe-btn vibe-git-stageall", "Stage all");
        stageAll.addEventListener("click", function () {
            state.files.forEach(function (f) { state.staged[f.file] = true; });
            renderStagedList();
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
        if (!state.files.length) {
            list.innerHTML = '<div class="vibe-empty">Working tree clean.</div>';
            return;
        }
        list.innerHTML = "";
        state.files.forEach(function (f) {
            var row = D.el("div", "vibe-list-item");
            var check = D.el("input", "vibe-git-check");
            check.type = "checkbox";
            check.checked = state.staged[f.file] !== false;
            check.addEventListener("change", function () {
                state.staged[f.file] = check.checked;
            });
            var label = D.el("span", "vibe-git-file", f.file);
            var badge = D.el("span", badgeClass(statusLetter(f)), statusLetter(f));
            row.appendChild(check);
            row.appendChild(label);
            row.appendChild(badge);
            row.addEventListener("click", function (ev) {
                if (ev.target === check) return;
                state.selected = f.file;
                renderCrumb();
                loadDiff();
            });
            list.appendChild(row);
        });
    }

    /* ── Data loaders ───────────────────────────────────────────────────── */
    function refreshAll() {
        loadStatus();
        loadLog();
        if (state.selected) loadDiff(); else loadTree();
    }

    function loadStatus() {
        D.api("/api/git/status?repo=" + encodeURIComponent(repoName()))
            .then(function (data) {
                state.files = (data && data.files) || [];
                state.branch = (data && data.branch) || null;
                var count = document.getElementById("vibeGitCount");
                if (count) {
                    count.textContent = state.files.length + " changed";
                    count.className = "vibe-status " + (state.files.length ? "warn" : "ok");
                }
                renderStagedList();
                if (!state.selected) loadTree();
                loadBranches();
            })
            .catch(function (err) {
                var list = document.getElementById("vibeGitFileList");
                if (list) list.innerHTML = '<div class="vibe-empty">Status error: ' + D.esc(err) + "</div>";
            });
    }

    function loadBranches() {
        D.api("/api/git/branches?repo=" + encodeURIComponent(repoName()))
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
                    opt.textContent = name + (name === state.branch ? " *" : "");
                    sel.appendChild(opt);
                });
                if (state.branch && names.indexOf(state.branch) !== -1) sel.value = state.branch;
            })
            .catch(function () { /* branch bar keeps its previous content */ });
    }

    function switchBranch(name) {
        if (!name) return;
        D.api("/api/git/branch/" + encodeURIComponent(name) + "?repo=" + encodeURIComponent(repoName()), {
            method: "POST",
        }).then(function (data) {
            if (data && data.status === "failure") {
                alert("Branch switch failed: " + ((data && data.error) || "unknown"));
            }
            state.selected = null;
            refreshAll();
        }).catch(function (err) { alert("Branch error: " + err); });
    }

    function loadLog() {
        D.api("/api/git/log?repo=" + encodeURIComponent(repoName()))
            .then(function (data) {
                state.log = (data && data.log) || (data && data.commits) || [];
            })
            .catch(function () { });
    }

    function pullBranch() {
        D.api("/api/git/pull?repo=" + encodeURIComponent(repoName()), { method: "POST" })
            .then(function (data) {
                if (data && data.status === "failure") {
                    alert("Pull failed: " + ((data && data.error) || "unknown"));
                } else {
                    refreshAll();
                }
            })
            .catch(function (err) { alert("Pull error: " + err); });
    }

    function pushBranch() {
        var btn = document.getElementById("vibeGitPushBtn");
        if (btn) { btn.disabled = true; btn.textContent = "Pushing…"; }
        D.api("/api/git/push?repo=" + encodeURIComponent(repoName()), { method: "POST" })
            .then(function (data) {
                if (data && (data.status === "failure" || data.success === false)) {
                    alert("Push failed: " + ((data && data.error) || "unknown"));
                } else {
                    loadStatus();
                    loadLog();
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
        var files = state.files
            .filter(function (f) { return state.staged[f.file] !== false; })
            .map(function (f) { return f.file; });
        D.api("/api/git/commit", {
            method: "POST",
            body: { message: msg.value.trim(), repo: repoName(), files: files },
        }).then(function (data) {
            if (data && (data.success === true || data.status === "success")) {
                msg.value = "";
                state.staged = {};
                if (andPush) pushBranch();
                refreshAll();
            } else {
                alert("Commit failed: " + ((data && data.error) || "unknown"));
            }
        }).catch(function (err) {
            alert("Commit error: " + err);
        }).finally(function () {
            if (btn) { btn.disabled = false; btn.textContent = "Commit"; }
        });
    }

    document.addEventListener("keydown", function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
            var msg = document.getElementById("vibeGitMessage");
            if (msg && msg === document.activeElement) commit(false);
        }
    });

    D.register("git", {
        build: function (body) {
            body.classList.add("vibe-git-dialog");
            body.appendChild(toolbar());
            var shell = D.el("div", "vibe-git-shell");
            shell.appendChild(center());
            shell.appendChild(sidebar());
            body.appendChild(shell);
            loadStatus();
            loadLog();
            loadTree();
        },
        teardown: function () {
            state = { files: [], tree: [], branch: null, path: "", selected: null, log: [], staged: {} };
        },
    });
})();
