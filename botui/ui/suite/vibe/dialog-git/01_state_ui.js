/* Vibe Source Control dialog — split from vibe/vibe-dialog-git.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__git = D.__git || {};
    var D = window.VibeDialogs;
    M.state = {
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
        M.state.files.forEach(function (f) { map[f.file] = M.statusLetter(f); });
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
            M.switchBranch(branchSel.value);
        });

        var fetchBtn = D.el("button", "vibe-btn", "↻ Fetch");
        fetchBtn.addEventListener("click", function () { M.refreshAll(); });

        var pullBtn = D.el("button", "vibe-btn", "⤓ Pull");
        pullBtn.addEventListener("click", M.pullBranch);

        var pushBtn = D.el("button", "vibe-btn", "⤒ Push");
        pushBtn.id = "vibeGitPushBtn";
        pushBtn.addEventListener("click", M.pushBranch);

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

    M.badgeClass = badgeClass;
    M.statusLetter = statusLetter;
    M.statusMap = statusMap;
    M.repoName = repoName;
    M.toolbar = toolbar;
    M.center = center;
})(window.VibeDialogs);
