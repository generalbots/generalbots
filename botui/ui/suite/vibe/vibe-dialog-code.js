/**
 * Vibe Code dialog — project workspace file browser + Monaco editor.
 * Lists/reads/writes the SELECTED project's workspace files via
 * /api/vibe/projects/:id/files (the real VIBE_WORKSPACE_ROOT output).
 *
 * File list is a FLAT list (not a tree): directories show a yellow folder
 * icon (SVG), files a document icon (SVG). Content is edited in
 * Monaco (vendored locally, syntax-highlighted by extension) with a plain
 * textarea fallback when Monaco is unavailable.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { files: [], current: null, savedContent: null, dirty: false, expandedFolders: {} };
    var monacoInstance = null;
    var monacoLang = "plaintext";
    var textarea = null;
    var editorHost = null;

    /* ── Inline SVG icons (stroke style, matches the Vibe shell toolbar) ── */
    var ICONS = {
        save: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>',
        newFile: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="12" x2="12" y2="18"/><line x1="9" y1="15" x2="15" y2="15"/></svg>',
        folder: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
        file: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="9" y1="13" x2="15" y2="13"/><line x1="9" y1="17" x2="13" y2="17"/></svg>',
    };

    function svgBtnIcon(key) {
        var svg = ICONS[key] || "";
        return '<span class="vibe-code-btn-icon" aria-hidden="true">' + svg + "</span>";
    }

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
        var save = D.el("button", "vibe-btn primary");
        save.innerHTML = svgBtnIcon("save") + '<span>Save (Ctrl+S)</span>';
        save.addEventListener("click", saveFile);
        var newBtn = D.el("button", "vibe-btn");
        newBtn.innerHTML = svgBtnIcon("newFile") + '<span>New File</span>';
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
        // #1395 — reset the dirty baseline whenever a file (re)loads.
        state.savedContent = getValue();
        setDirty(false);
    }

    // #1395 — dirty-state guard: unsaved edits show ● in the file chip and
    // block silent overwrite when switching files or closing the dialog.
    // #1395 — status line doubles as error surface; errors get a distinct
    // class and stay visible until the next action replaces them.
    function setStatus(msg, isError) {
        var status = document.getElementById("vibeCodeStatusMsg");
        if (!status) return;
        status.textContent = msg;
        status.className = isError ? "vibe-code-status-error" : "";
    }

    function isDirty() {
        return state.current != null && getValue() !== state.savedContent;
    }

    function setDirty(dirty) {
        var chip = document.getElementById("vibeCodeFileName");
        if (chip) {
            var base = displayName(state.current || "");
            chip.textContent = dirty ? base + " \u25CF" : base;
        }
        state.dirty = !!dirty;
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
        // #1395 — mark the chip dirty as soon as the user edits.
        var markDirty = function () { setDirty(true); };
        if (monacoInstance) {
            monacoInstance.onDidChangeModelContent(markDirty);
        } else {
            textarea.removeEventListener("input", markDirty);
            textarea.addEventListener("input", markDirty);
        }
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
        return '<span class="vibe-code-tree-icon vibe-code-tree-folder" aria-hidden="true">' + ICONS.folder + "</span>";
    }

    function fileIconEl() {
        return '<span class="vibe-code-tree-icon vibe-code-tree-file" aria-hidden="true">' + ICONS.file + "</span>";
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
            // #1395 — preserve expansion state across loadFiles() re-renders
            // (after save / project switch) keyed by full folder path.
            var folderPath = node._path ? node._path + "/" + key : key;
            row.setAttribute("data-folder", folderPath);
            var expanded = !!state.expandedFolders[folderPath];
            var childWrap = D.el("div", "vibe-tree-children");
            childWrap.style.display = expanded ? "block" : "none";
            if (expanded) arrow.textContent = "▾";
            row.addEventListener("click", function () {
                expanded = !expanded;
                state.expandedFolders[folderPath] = expanded;
                arrow.textContent = expanded ? "▾" : "▸";
                childWrap.style.display = expanded ? "block" : "none";
            });
            list.appendChild(row);
            list.appendChild(childWrap);
            // Render the folder's descendants directly into its child block;
            // collapsing the folder hides the whole block at once.
            child._path = folderPath;
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
        if (isDirty()) {
            confirmDiscard(function () { openFile(name); });
            return;
        }
        state.current = name;
        var nameEl = document.getElementById("vibeCodeFileName");
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
            setStatus((data && data.success) ? "loaded " + displayName(name) : "error loading " + displayName(name) + ": " + ((data && data.error) || "failed"), !(data && data.success));
            highlightActive();
        }).catch(function (err) {
            setValue("");
            setStatus("error loading " + displayName(name) + ": " + err, true);
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
        if (!pid) {
            setStatus("select a project first", true);
            return;
        }
        var doSave = function (path) {
            D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", {
                method: "POST",
                body: { path: path, content: getValue() },
            }).then(function (data) {
                setStatus((data && data.success) ? "saved " + displayName(path) : "save: " + ((data && data.error) || "failed"), !(data && data.success));
                if (data && data.success) {
                    state.savedContent = getValue();
                    setDirty(false);
                }
                loadFiles();
            }).catch(function (err) {
                setStatus("save error: " + err, true);
            });
        };
        if (!state.current) {
            // #1395 — in-dialog input instead of native prompt() (blocked in
            // desktop-shell contexts).
            if (window.WindowManager && window.WindowManager.promptFloating) {
                window.WindowManager.promptFloating("Save file", "File name to save (workspace root ok):", "", function (name) {
                    if (!name || !name.trim()) return;
                    state.current = name.trim();
                    var nameEl = document.getElementById("vibeCodeFileName");
                    if (nameEl) {
                        nameEl.textContent = displayName(state.current);
                        nameEl.className = "vibe-status ok";
                    }
                    doSave(state.current);
                });
            } else {
                setStatus("no file open — use New File first", true);
            }
            return;
        }
        doSave(state.current);
    }

    function newFile() {
        var pid = selectedProjectId();
        if (!pid) return;
        if (isDirty()) {
            confirmDiscard(function () { newFile(); });
            return;
        }
        var create = function (name) {
            if (!name || !name.trim()) return;
            name = name.trim();
            if (state.files.indexOf(name) === -1) state.files.push(name);
            state.current = name;
            var nameEl = document.getElementById("vibeCodeFileName");
            setValue("");
            if (nameEl) {
                nameEl.textContent = displayName(name);
                nameEl.className = "vibe-status ok";
            }
            setStatus("new file " + displayName(name) + " (Save to create)");
            showMonacoFor(name);
        };
        if (window.WindowManager && window.WindowManager.promptFloating) {
            window.WindowManager.promptFloating("New file", "File name:", "", create);
        } else {
            setStatus("cannot open name input (window manager unavailable)", true);
        }
    }

    // #1395 — ask before discarding unsaved edits; floating confirm, no native modals.
    function confirmDiscard(then) {
        if (window.WindowManager && window.WindowManager.confirmFloating) {
            window.WindowManager.confirmFloating("Unsaved changes", "Discard unsaved edits to " + displayName(state.current || "") + "?", function () {
                setDirty(false);
                state.savedContent = getValue();
                then();
            }, null, "Discard");
        } else {
            then();
        }
    }

    // Reload the file list when the user selects a different project in the
    // sidebar, so the editor always reflects the active project's workspace.
    document.addEventListener("gb:vibe-project", function () {
        state = { files: [], current: null, savedContent: null, dirty: false, expandedFolders: {} };
        loadFiles();
    });

    document.addEventListener("keydown", function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            // #1395 — Ctrl+S works whenever focus lives anywhere inside the
            // code editor dialog (chip, tree, status bar), not only the editor.
            var dlg = document.querySelector("[id^=window-body-] .vibe-code-editor")
                || document.getElementById("vibeMonacoHost")
                || document.getElementById("vibeCodeFileList");
            var active = document.activeElement;
            var insideDialog = dlg && active && dlg.contains(active);
            if (insideDialog || (monacoInstance && monacoInstance.hasTextFocus && monacoInstance.hasTextFocus()) || (textarea && textarea === active)) {
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
            if (isDirty()) {
                confirmDiscard(function () { });
            }
            state = { files: [], current: null, savedContent: null, dirty: false, expandedFolders: {} };
            if (monacoInstance) {
                try { monacoInstance.dispose(); } catch (ignore) { }
                monacoInstance = null;
            }
            if (textarea) textarea.value = "";
        },
    });
})();
