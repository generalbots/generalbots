/* Vibe Code dialog — split from vibe/vibe-dialog-code.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__code;
    var D = window.VibeDialogs;
    /* ------------------------------------------------- file ops */

    function displayName(path) {
        var clean = String(path).replace(/\/+$/, "");
        var segs = clean.split("/");
        return segs[segs.length - 1] || clean;
    }

    // Build a hierarchical tree from the backend's recursive flat paths
    // (`src/`, `src/main.py`, `index.js`). Returns a root node whose
    // `folders` map holds subdirectories and whose `files` array holds the
    // full relative path of each file leaf.
    function buildTree(paths) {
        var root = { folders: {}, files: [] };
        paths.forEach(function (p) {
            var isDir = /\/$/.test(p);
            var clean = String(p).replace(/\/+$/, "");
            if (!clean) return;
            var parts = clean.split("/");
            var node = root;
            for (var i = 0; i < parts.length; i++) {
                var part = parts[i];
                var last = i === parts.length - 1;
                if (last && !isDir) {
                    node.files.push({ path: clean, name: part });
                } else {
                    if (!node.folders[part]) {
                        node.folders[part] = { name: part, folders: {}, files: [] };
                    }
                    node = node.folders[part];
                }
            }
        });
        return root;
    }

    function folderIcon() {
        return '<span class="vibe-code-tree-icon vibe-code-tree-folder" aria-hidden="true">' + M.ICONS.folder + "</span>";
    }

    function fileIconEl() {
        return '<span class="vibe-code-tree-icon vibe-code-tree-file" aria-hidden="true">' + M.ICONS.file + "</span>";
    }

    function sortTree(node) {
        var folders = Object.keys(node.folders).sort();
        node.files.sort(function (a, b) { return a.name.localeCompare(b.name); });
        folders.forEach(function (k) { M.sortTree(node.folders[k]); });
        node._sortedFolders = folders;
    }

    function renderTree(node, list, depth) {
        depth = depth || 0;
        (node._sortedFolders || Object.keys(node.folders).sort()).forEach(function (key) {
            var child = node.folders[key];
            var indent = depth * 14;
            var row = D.el("div", "vibe-code-file vibe-code-folder");
            row.style.paddingLeft = (6 + indent) + "px";
            row.innerHTML = '<span class="vibe-arrow">▸</span>' + M.folderIcon() +
                '<span class="vibe-file-name">' + D.esc(key) + "</span>";
            var arrow = row.querySelector(".vibe-arrow");
            // #1395 — preserve expansion state across loadFiles() re-renders
            // (after save / project switch) keyed by full folder path.
            var folderPath = node._path ? node._path + "/" + key : key;
            row.setAttribute("data-folder", folderPath);
            var expanded = !!M.state.expandedFolders[folderPath];
            var childWrap = D.el("div", "vibe-tree-children");
            childWrap.style.display = expanded ? "block" : "none";
            if (expanded) arrow.textContent = "▾";
            row.addEventListener("click", function () {
                expanded = !expanded;
                M.state.expandedFolders[folderPath] = expanded;
                arrow.textContent = expanded ? "▾" : "▸";
                childWrap.style.display = expanded ? "block" : "none";
            });
            list.appendChild(row);
            list.appendChild(childWrap);
            // Render the folder's descendants directly into its child block;
            // collapsing the folder hides the whole block at once.
            child._path = folderPath;
            M.renderTree(child, childWrap, depth + 1);
        });
        node.files.forEach(function (f) {
            var indent = depth * 14;
            var row = D.el("div", "vibe-code-file");
            row.style.paddingLeft = (6 + indent + 16) + "px";
            if (M.state.current === f.path) row.classList.add("active");
            row.innerHTML = M.fileIconEl() +
                '<span class="vibe-file-name" title="' + D.esc(f.path) + '">' + D.esc(f.name) + "</span>";
            row.addEventListener("click", function () { M.openFile(f.path); });
            list.appendChild(row);
        });
    }

    M.displayName = displayName;
    M.buildTree = buildTree;
    M.folderIcon = folderIcon;
    M.fileIconEl = fileIconEl;
    M.sortTree = sortTree;
    M.renderTree = renderTree;
})(window.VibeDialogs);
