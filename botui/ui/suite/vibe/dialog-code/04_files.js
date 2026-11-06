/* Vibe Code dialog — split from vibe/vibe-dialog-code.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__code;
    var D = window.VibeDialogs;
    function loadFiles() {
        var list = document.getElementById("vibeCodeFileList");
        if (!list) return;
        var pid = M.selectedProjectId();
        if (!pid) {
            list.innerHTML = '<div class="vibe-empty">Select a project first.</div>';
            M.state.files = [];
            return;
        }
        D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files").then(function (data) {
            M.state.files = (data && data.files) || [];
            if (!list) return;
            if (!M.state.files.length) {
                list.innerHTML = '<div class="vibe-empty">Empty workspace.</div>';
                return;
            }
            var tree = M.buildTree(M.state.files);
            M.sortTree(tree);
            list.innerHTML = "";
            M.renderTree(tree, list);
        }).catch(function (err) {
            if (list) list.innerHTML = '<div class="vibe-empty">Error: ' + D.esc(err) + "</div>";
        });
    }

    function openFile(name) {
        var pid = M.selectedProjectId();
        if (!pid) return;
        if (M.isDirty()) {
            M.confirmDiscard(function () { M.openFile(name); });
            return;
        }
        M.state.current = name;
        var nameEl = document.getElementById("vibeCodeFileName");
        if (nameEl) {
            nameEl.textContent = M.displayName(name);
            nameEl.className = "vibe-status ok";
        }
        M.setValue("Loading...");
        D.api(
            "/api/vibe/projects/" + encodeURIComponent(pid) + "/files/content?path=" + encodeURIComponent(name),
        ).then(function (data) {
            var content = (data && data.content != null) ? String(data.content) : "";
            M.setValue(content);
            M.setStatus((data && data.success) ? "loaded " + M.displayName(name) : "error loading " + M.displayName(name) + ": " + ((data && data.error) || "failed"), !(data && data.success));
            M.highlightActive();
        }).catch(function (err) {
            M.setValue("");
            M.setStatus("error loading " + M.displayName(name) + ": " + err, true);
        });
        M.showMonacoFor(name);
    }

    function highlightActive() {
        var list = document.getElementById("vibeCodeFileList");
        if (!list) return;
        Array.prototype.forEach.call(list.children, function (row) {
            var nameEl = row.querySelector(".vibe-file-name");
            var active = nameEl && nameEl.getAttribute("title") === M.displayName(M.state.current);
            row.classList.toggle("active", !!active);
        });
    }

    function saveFile() {
        var pid = M.selectedProjectId();
        if (!pid) {
            M.setStatus("select a project first", true);
            return;
        }
        var doSave = function (path) {
            D.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", {
                method: "POST",
                body: { path: path, content: M.getValue() },
            }).then(function (data) {
                M.setStatus((data && data.success) ? "saved " + M.displayName(path) : "save: " + ((data && data.error) || "failed"), !(data && data.success));
                if (data && data.success) {
                    M.state.savedContent = M.getValue();
                    M.setDirty(false);
                }
                M.loadFiles();
            }).catch(function (err) {
                M.setStatus("save error: " + err, true);
            });
        };
        if (!M.state.current) {
            // #1395 — in-dialog input instead of native prompt() (blocked in
            // desktop-shell contexts).
            if (window.WindowManager && window.WindowManager.promptFloating) {
                window.WindowManager.promptFloating("Save file", "File name to save (workspace root ok):", "", function (name) {
                    if (!name || !name.trim()) return;
                    M.state.current = name.trim();
                    var nameEl = document.getElementById("vibeCodeFileName");
                    if (nameEl) {
                        nameEl.textContent = M.displayName(M.state.current);
                        nameEl.className = "vibe-status ok";
                    }
                    doSave(M.state.current);
                });
            } else {
                M.setStatus("no file open — use New File first", true);
            }
            return;
        }
        doSave(M.state.current);
    }

    function newFile() {
        var pid = M.selectedProjectId();
        if (!pid) return;
        if (M.isDirty()) {
            M.confirmDiscard(function () { M.newFile(); });
            return;
        }
        var create = function (name) {
            if (!name || !name.trim()) return;
            name = name.trim();
            if (M.state.files.indexOf(name) === -1) M.state.files.push(name);
            M.state.current = name;
            var nameEl = document.getElementById("vibeCodeFileName");
            M.setValue("");
            if (nameEl) {
                nameEl.textContent = M.displayName(name);
                nameEl.className = "vibe-status ok";
            }
            M.setStatus("new file " + M.displayName(name) + " (Save to create)");
            M.showMonacoFor(name);
        };
        if (window.WindowManager && window.WindowManager.promptFloating) {
            window.WindowManager.promptFloating("New file", "File name:", "", create);
        } else {
            M.setStatus("cannot open name input (window manager unavailable)", true);
        }
    }

    // #1395 — ask before discarding unsaved edits; floating confirm, no native modals.
    function confirmDiscard(then) {
        if (window.WindowManager && window.WindowManager.confirmFloating) {
            window.WindowManager.confirmFloating("Unsaved changes", "Discard unsaved edits to " + M.displayName(M.state.current || "") + "?", function () {
                M.setDirty(false);
                M.state.savedContent = M.getValue();
                then();
            }, null, "Discard");
        } else {
            then();
        }
    }

    M.loadFiles = loadFiles;
    M.openFile = openFile;
    M.highlightActive = highlightActive;
    M.saveFile = saveFile;
    M.newFile = newFile;
    M.confirmDiscard = confirmDiscard;
})(window.VibeDialogs);
