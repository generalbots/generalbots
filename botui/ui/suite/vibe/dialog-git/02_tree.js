/* Vibe Source Control dialog — split from vibe/vibe-dialog-git.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__git;
    var D = window.VibeDialogs;
    function renderCrumb() {
        var crumb = document.getElementById("vibeGitCrumb");
        if (!crumb) return;
        crumb.innerHTML = "";
        if (!M.state.selected) {
            var repoLink = D.el("span", "vibe-git-crumb-link", M.repoName());
            repoLink.addEventListener("click", function () { M.openTree(""); });
            crumb.appendChild(repoLink);
            if (M.state.path) {
                var parts = M.state.path.split("/");
                var acc = "";
                parts.forEach(function (part) {
                    acc = acc ? acc + "/" + part : part;
                    crumb.appendChild(D.el("span", "vibe-git-crumb-sep", " / "));
                    var seg = acc;
                    var link = D.el("span", "vibe-git-crumb-link", part);
                    link.addEventListener("click", function () { M.openTree(seg); });
                    crumb.appendChild(link);
                });
            }
        } else {
            var back = D.el("span", "vibe-git-crumb-link", "← " + M.repoName());
            back.addEventListener("click", function () {
                M.state.selected = null;
                M.openTree(M.state.path);
            });
            crumb.appendChild(back);
            crumb.appendChild(D.el("span", "vibe-git-crumb-sep", " / "));
            crumb.appendChild(D.el("span", "vibe-git-crumb-file", M.state.selected));
        }
    }

    function openTree(path) {
        M.state.path = path || "";
        M.state.selected = null;
        M.renderCrumb();
        M.loadTree();
    }

    function loadTree() {
        var grid = document.getElementById("vibeGitMain");
        if (grid) grid.innerHTML = '<div class="vibe-empty">Loading tree…</div>';
        D.api("/api/git/tree?repo=" + encodeURIComponent(M.repoName()) +
            "&path=" + encodeURIComponent(M.state.path))
            .then(function (data) {
                M.state.tree = (data && data.entries) || [];
                M.renderTree();
            })
            .catch(function (err) {
                if (grid) grid.innerHTML = '<div class="vibe-empty">Tree error: ' + D.esc(err) + "</div>";
            });
    }

    function renderTree() {
        var grid = document.getElementById("vibeGitMain");
        if (!grid) return;
        var map = M.statusMap();
        if (!M.state.tree.length) {
            grid.innerHTML = '<div class="vibe-empty">Empty directory.</div>';
            return;
        }
        var list = D.el("div", "vibe-git-list");
        M.state.tree.forEach(function (entry) {
            var row = D.el("div", "vibe-git-row" + (entry.is_dir ? " is-dir" : ""));
            var icon = D.el("span", "vibe-git-icon", entry.is_dir ? "📁" : "📄");
            var name = D.el("span", "vibe-git-name", M.fileName(entry.path));
            row.appendChild(icon);
            row.appendChild(name);
            if (map[entry.path]) {
                row.appendChild(D.el("span", M.badgeClass(map[entry.path]), map[entry.path]));
            }
            row.addEventListener("click", function () {
                if (entry.is_dir) {
                    M.openTree(entry.path);
                } else {
                    M.state.selected = entry.path;
                    M.renderCrumb();
                    M.loadDiff();
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
        if (!M.state.selected) {
            M.loadTree();
            return;
        }
        M.renderCrumb();
        if (grid) grid.innerHTML = '<div class="vibe-empty">Loading diff…</div>';
        D.api("/api/git/diff/" + encodeURIComponent(M.state.selected) + "?repo=" +
            encodeURIComponent(M.repoName()))
            .then(function (data) {
                if (data && data.diff != null && String(data.diff).trim()) {
                    grid.innerHTML = '<div class="vibe-diff"><pre>' +
                        M.highlightDiff(String(data.diff)) + "</pre></div>";
                } else if (data && data.error) {
                    grid.innerHTML = '<div class="vibe-empty">' + D.esc(data.error) + "</div>";
                } else {
                    grid.innerHTML = '<div class="vibe-empty">No textual changes (binary or untracked file).</div>';
                }
            })
            .catch(function () { M.loadTree(); });
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

    M.renderCrumb = renderCrumb;
    M.openTree = openTree;
    M.loadTree = loadTree;
    M.renderTree = renderTree;
    M.fileName = fileName;
    M.loadDiff = loadDiff;
    M.highlightDiff = highlightDiff;
})(window.VibeDialogs);
