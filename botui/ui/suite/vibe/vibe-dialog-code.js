/**
 * Vibe Code dialog — project workspace file browser + Monaco editor.
 * Lists/reads/writes the SELECTED project's workspace files via
 * /api/vibe/projects/:id/files (the real VIBE_WORKSPACE_ROOT output).
 *
 * File list is a FLAT list (not a tree): directories show a yellow folder
 * icon (📁), files a Windows-style document icon (📄). Content is edited in
 * Monaco (vendored locally, syntax-highlighted by extension) with a plain
 * textarea fallback when Monaco is unavailable.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { files: [], current: null };
    var monacoInstance = null;
    var monacoLang = "plaintext";
    var textarea = null;
    var editorHost = null;

    function selectedProjectId() {
        return typeof window.currentProjectId !== "undefined" && window.currentProjectId
            ? window.currentProjectId
            : null;
    }

    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        var head = D.el("div", "vibe-dialog-title");
        head.textContent = "FILES";
        head.style.padding = "10px";

        var list = D.el("div", "vibe-list");
        list.id = "vibeCodeFileList";
        list.innerHTML = '<div class="vibe-empty">Loading workspace...</div>';

        box.appendChild(head);
        box.appendChild(list);
        return box;
    }

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";
        box.appendChild(editorArea());
        box.appendChild(statusBar());
        return box;
    }

    function editorArea() {
        var wrap = D.el("div", "vibe-code-editor");
        wrap.id = "vibeCodeEditorWrap";
        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var name = D.el("span", "vibe-status info", "no file open");
        name.id = "vibeCodeFileName";
        var lang = D.el("span", "vibe-status warn");
        lang.id = "vibeCodeFileLang";
        lang.style.marginLeft = "6px";
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var save = D.el("button", "vibe-btn primary", "💾 Save (Ctrl+S)");
        save.addEventListener("click", saveFile);
        var newBtn = D.el("button", "vibe-btn", "New File");
        newBtn.addEventListener("click", newFile);
        toolbar.appendChild(name);
        toolbar.appendChild(lang);
        toolbar.appendChild(spacer);
        toolbar.appendChild(newBtn);
        toolbar.appendChild(save);
        wrap.appendChild(toolbar);

        // Monaco host — replaced by the textarea fallback if Monaco fails.
        editorHost = D.el("div", "vibe-code-monaco");
        editorHost.id = "vibeMonacoHost";
        editorHost.style.cssText = "flex:1;min-height:0;display:none;";
        wrap.appendChild(editorHost);

        textarea = D.el("textarea", "vibe-textarea");
        textarea.id = "vibeCodeContent";
        textarea.placeholder = "Open a file from the workspace to edit it.\n\nFiles live in the project workspace (VIBE_WORKSPACE_ROOT).";
        textarea.spellcheck = false;
        textarea.style.display = "none";
        wrap.appendChild(textarea);
        return wrap;
    }

    function statusBar() {
        var bar = D.el("div", "vibe-browser-status");
        bar.id = "vibeCodeStatus";
        bar.innerHTML = '<span id="vibeCodeStatusMsg">editor ready</span>';
        return bar;
    }

    /* ------------------------------------------------- Monaco */

    // Language per file extension (Monaco registered language ids).
    function langFor(name) {
        var ext = String(name).split(".").pop().toLowerCase();
        var map = {
            rs: "rust", py: "python", js: "javascript", mjs: "javascript",
            ts: "typescript", tsx: "typescript", jsx: "javascript",
            html: "html", htm: "html", css: "css", scss: "scss",
            json: "json", md: "markdown", xml: "xml", yml: "yaml", yaml: "yaml",
            sh: "shell", bash: "shell", sql: "sql", java: "java",
            c: "c", h: "c", cpp: "cpp", hpp: "cpp", cs: "csharp",
            go: "go", rb: "ruby", php: "php", kt: "kotlin",
            toml: "ini", ini: "ini", txt: "plaintext", csv: "plaintext",
        };
        return map[ext] || "plaintext";
    }

    function loadMonaco(cb) {
        if (window.monaco) { cb(); return; }
        if (document.getElementById("vibe-monaco-script")) {
            // Already loading; poll until ready.
            var tries = 0;
            var t = setInterval(function () {
                tries++;
                if (window.monaco) { clearInterval(t); cb(); }
                else if (tries > 100) { clearInterval(t); useTextareaFallback(); }
            }, 100);
            return;
        }
        var script = document.createElement("script");
        script.id = "vibe-monaco-script";
        script.src = "/suite/js/vendor/vs/loader.js";
        script.onload = function () {
            require.config({ paths: { "vs": "/suite/js/vendor/vs" } });
            require(["vs/editor/editor.main"], function () { cb(); });
        };
        script.onerror = function () { useTextareaFallback(); };
        document.head.appendChild(script);
    }

    function useTextareaFallback() {
        if (!editorHost || !textarea) return;
        editorHost.style.display = "none";
        textarea.style.display = "block";
    }

    function setValue(content) {
        if (monacoInstance) {
            monacoInstance.setValue(content == null ? "" : String(content));
        } else {
            textarea.value = content == null ? "" : String(content);
        }
    }

    function getValue() {
        if (monacoInstance) return monacoInstance.getValue();
        return textarea.value;
    }

    function showMonacoFor(name) {
        monacoLang = langFor(name);
        var langEl = document.getElementById("vibeCodeFileLang");
        if (langEl) {
            langEl.textContent = monacoLang.toUpperCase();
            langEl.className = "vibe-status warn";
        }
        if (!window.monaco || !editorHost) { useTextareaFallback(); return; }
        editorHost.style.display = "block";
        textarea.style.display = "none";
        if (monacoInstance) {
            monacoInstance.dispose();
            monacoInstance = null;
        }
        monacoInstance = window.monaco.editor.create(editorHost, {
            value: "",
            language: monacoLang,
            theme: "vs-dark",
            fontSize: 12,
            fontFamily: '"Fira Code", Consolas, monospace',
            automaticLayout: true,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            tabSize: 4,
        });
        if (textarea.value) monacoInstance.setValue(textarea.value);
    }

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
        return '<span style="color:#eab308;filter:drop-shadow(0 0 2px rgba(234,179,8,0.4))">📁</span>';
    }

    function fileIconEl() {
        return '<span style="color:#9aa4b2">📄</span>';
    }

    function sortTree(node) {
        var folders = Object.keys(node.folders).sort();
        node.files.sort(function (a, b) { return a.name.localeCompare(b.name); });
        folders.forEach(function (k) { sortTree(node.folders[k]); });
        node._sortedFolders = folders;
    }

    function renderTree(node, list, depth) {
        depth = depth || 0;
        (node._sortedFolders || Object.keys(node.folders).sort()).forEach(function (key) {
            var child = node.folders[key];
            var indent = depth * 14;
            var row = D.el("div", "vibe-code-file vibe-code-folder");
            row.style.paddingLeft = (6 + indent) + "px";
            row.innerHTML = '<span class="vibe-arrow">▸</span>' + folderIcon() +
                '<span class="vibe-file-name">' + D.esc(key) + "</span>";
            var arrow = row.querySelector(".vibe-arrow");
            var expanded = false;
            var childWrap = D.el("div", "vibe-tree-children");
            childWrap.style.display = "none";
            row.addEventListener("click", function () {
                expanded = !expanded;
                arrow.textContent = expanded ? "▾" : "▸";
                childWrap.style.display = expanded ? "block" : "none";
            });
            list.appendChild(row);
            list.appendChild(childWrap);
            // Render the folder's descendants directly into its child block;
            // collapsing the folder hides the whole block at once.
            renderTree(child, childWrap, depth + 1);
        });
        node.files.forEach(function (f) {
            var indent = depth * 14;
            var row = D.el("div", "vibe-code-file");
            row.style.paddingLeft = (6 + indent + 16) + "px";
            if (state.current === f.path) row.classList.add("active");
            row.innerHTML = fileIconEl() +
                '<span class="vibe-file-name" title="' + D.esc(f.path) + '">' + D.esc(f.name) + "</span>";
            row.addEventListener("click", function () { openFile(f.path); });
            list.appendChild(row);
        });
    }

    function loadFiles() {
        var list = document.getElementById("vibeCodeFileList");
        if (!list) return;
        var pid = selectedProjectId();
        if (!pid) {
            list.innerHTML = '<div class="vibe-empty">Select a project first.</div>';
            state.files = [];
            return;
        }
        D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files").then(function (data) {
            state.files = (data && data.files) || [];
            if (!list) return;
            if (!state.files.length) {
                list.innerHTML = '<div class="vibe-empty">Empty workspace.</div>';
                return;
            }
            var tree = buildTree(state.files);
            sortTree(tree);
            list.innerHTML = "";
            renderTree(tree, list);
        }).catch(function (err) {
            if (list) list.innerHTML = '<div class="vibe-empty">Error: ' + D.esc(err) + "</div>";
        });
    }

    function openFile(name) {
        var pid = selectedProjectId();
        if (!pid) return;
        state.current = name;
        var nameEl = document.getElementById("vibeCodeFileName");
        var status = document.getElementById("vibeCodeStatusMsg");
        if (nameEl) {
            nameEl.textContent = displayName(name);
            nameEl.className = "vibe-status ok";
        }
        setValue("Loading...");
        D.api(
            "/api/vibe/projects/" + encodeURIComponent(pid) + "/files/content?path=" + encodeURIComponent(name),
        ).then(function (data) {
            var content = (data && data.content != null) ? String(data.content) : "";
            setValue(content);
            if (status) status.textContent = (data && data.success) ? "loaded " + displayName(name) : "error loading " + displayName(name) + ": " + ((data && data.error) || "failed");
            highlightActive();
        }).catch(function (err) {
            setValue("");
            if (status) status.textContent = "error loading " + displayName(name) + ": " + err;
        });
        showMonacoFor(name);
    }

    function highlightActive() {
        var list = document.getElementById("vibeCodeFileList");
        if (!list) return;
        Array.prototype.forEach.call(list.children, function (row) {
            var nameEl = row.querySelector(".vibe-file-name");
            var active = nameEl && nameEl.getAttribute("title") === displayName(state.current);
            row.classList.toggle("active", !!active);
        });
    }

    function saveFile() {
        var pid = selectedProjectId();
        var status = document.getElementById("vibeCodeStatusMsg");
        if (!pid) {
            if (status) status.textContent = "select a project first";
            return;
        }
        if (!state.current) {
            var name = prompt("File name to save (workspace root ok):");
            if (!name) return;
            state.current = name.trim();
        }
        D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", {
            method: "POST",
            body: { path: state.current, content: getValue() },
        }).then(function (data) {
            if (status) {
                status.textContent = (data && data.success) ? "saved " + displayName(state.current) : "save: " + ((data && data.error) || "failed");
            }
            loadFiles();
        }).catch(function (err) {
            if (status) status.textContent = "save error: " + err;
        });
    }

    function newFile() {
        var pid = selectedProjectId();
        if (!pid) return;
        var name = prompt("New file name:");
        if (!name) return;
        name = name.trim();
        if (!name) return;
        if (state.files.indexOf(name) === -1) state.files.push(name);
        state.current = name;
        var nameEl = document.getElementById("vibeCodeFileName");
        var status = document.getElementById("vibeCodeStatusMsg");
        setValue("");
        if (nameEl) {
            nameEl.textContent = displayName(name);
            nameEl.className = "vibe-status ok";
        }
        if (status) status.textContent = "new file " + displayName(name) + " (Save to create)";
        showMonacoFor(name);
    }

    // Reload the file list when the user selects a different project in the
    // sidebar, so the editor always reflects the active project's workspace.
    document.addEventListener("gb:vibe-project", function () {
        state = { files: [], current: null };
        loadFiles();
    });

    document.addEventListener("keydown", function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            var activeIsEditor = monacoInstance
                ? true
                : (textarea && textarea === document.activeElement);
            if (activeIsEditor) {
                e.preventDefault();
                saveFile();
            }
        }
    });

    D.register("code", {
        build: function (body) {
            body.appendChild(sidebar());
            body.appendChild(main());
            loadMonaco(function () {
                showMonacoFor(state.current || "untitled.txt");
            });
            loadFiles();
        },
        teardown: function () {
            state = { files: [], current: null };
            if (monacoInstance) {
                try { monacoInstance.dispose(); } catch (ignore) { }
                monacoInstance = null;
            }
            if (textarea) textarea.value = "";
        },
    });
})();
