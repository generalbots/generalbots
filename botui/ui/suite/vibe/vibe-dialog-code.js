/**
 * Vibe Code dialog — workspace file browser + editor.
 * Lists files via /api/editor/files, reads via
 * /api/editor/file/:path, saves via POST /api/editor/save
 * (name=path, content=body) — matching boteditor handlers.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { files: [], current: null };

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
        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var name = D.el("span", "vibe-status info", "no file open");
        name.id = "vibeCodeFileName";
        var spacer = D.el("span");
        spacer.style.flex = "1";
        var save = D.el("button", "vibe-btn primary", "💾 Save (Ctrl+S)");
        save.addEventListener("click", saveFile);
        var newBtn = D.el("button", "vibe-btn", "New File");
        newBtn.addEventListener("click", newFile);
        toolbar.appendChild(name);
        toolbar.appendChild(spacer);
        toolbar.appendChild(newBtn);
        toolbar.appendChild(save);
        wrap.appendChild(toolbar);
        var ta = D.el("textarea", "vibe-textarea");
        ta.id = "vibeCodeContent";
        ta.placeholder = "Open a file from the workspace to edit it.\n\nFiles live in the project workspace (VIBE_WORKSPACE_ROOT).";
        ta.spellcheck = false;
        wrap.appendChild(ta);
        return wrap;
    }

    function statusBar() {
        var bar = D.el("div", "vibe-browser-status");
        bar.id = "vibeCodeStatus";
        bar.innerHTML = '<span id="vibeCodeStatusMsg">editor ready</span>';
        return bar;
    }

    function loadFiles() {
        var list = document.getElementById("vibeCodeFileList");
        D.api("/api/editor/files").then(function (data) {
            state.files = (data && data.files) || [];
            if (!list) return;
            if (!state.files.length) {
                list.innerHTML = '<div class="vibe-empty">Empty workspace.</div>';
                return;
            }
            list.innerHTML = "";
            state.files.forEach(function (f) {
                var row = D.el("div", "vibe-code-file");
                if (state.current === f) row.classList.add("active");
                row.textContent = "📄 " + f;
                row.addEventListener("click", function () { openFile(f); });
                list.appendChild(row);
            });
        }).catch(function (err) {
            if (list) list.innerHTML = '<div class="vibe-empty">Error: ' + D.esc(err) + "</div>";
        });
    }

    function openFile(name) {
        state.current = name;
        var ta = document.getElementById("vibeCodeContent");
        var nameEl = document.getElementById("vibeCodeFileName");
        var status = document.getElementById("vibeCodeStatusMsg");
        if (ta) ta.value = "Loading...";
        if (nameEl) {
            nameEl.textContent = name;
            nameEl.className = "vibe-status ok";
        }
        D.api("/api/editor/file/" + encodeURIComponent(name)).then(function (data) {
            if (ta) ta.value = (data && data.content != null) ? String(data.content) : "";
            if (status) status.textContent = "loaded " + name;
        }).catch(function (err) {
            if (ta) ta.value = "";
            if (status) status.textContent = "error loading " + name + ": " + err;
        });
    }

    function saveFile() {
        var ta = document.getElementById("vibeCodeContent");
        var status = document.getElementById("vibeCodeStatusMsg");
        if (!state.current) {
            var name = prompt("File name to save (workspace root ok):");
            if (!name) return;
            state.current = name.trim();
        }
        D.api("/api/editor/save", {
            method: "POST",
            body: { name: state.current, content: ta ? ta.value : "" },
        }).then(function (data) {
            if (status) {
                status.textContent = (data && data.success) ? "saved " + state.current : "save: " + ((data && data.error) || "failed");
            }
            loadFiles();
        }).catch(function (err) {
            if (status) status.textContent = "save error: " + err;
        });
    }

    function newFile() {
        var name = prompt("New file name:");
        if (!name) return;
        if (state.files.indexOf(name) === -1) state.files.push(name);
        openFile(name);
        loadFiles();
    }

    document.addEventListener("keydown", function (e) {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            var ta = document.getElementById("vibeCodeContent");
            if (ta && ta === document.activeElement) {
                e.preventDefault();
                saveFile();
            }
        }
    });

    D.register("code", {
        build: function (body) {
            body.appendChild(sidebar());
            body.appendChild(main());
            loadFiles();
        },
        teardown: function () {
            state = { files: [], current: null };
        },
    });
})();