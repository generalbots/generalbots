/* Vibe Code dialog — split from vibe/vibe-dialog-code.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__code;
    var D = window.VibeDialogs;
    // Reload the file list when the user selects a different project in the
    // sidebar, so the editor always reflects the active project's workspace.
    document.addEventListener("gb:vibe-project", function () {
        M.state = { files: [], current: null, savedContent: null, dirty: false, expandedFolders: {} };
        M.loadFiles();
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
            if (insideDialog || (M.monacoInstance && M.monacoInstance.hasTextFocus && M.monacoInstance.hasTextFocus()) || (M.textarea && M.textarea === active)) {
                e.preventDefault();
                M.saveFile();
            }
        }
    });

    D.register("code", {
        build: function (body) {
            body.appendChild(M.sidebar());
            body.appendChild(M.main());
            M.loadMonaco(function () {
                M.showMonacoFor(M.state.current || "untitled.txt");
            });
            M.loadFiles();
        },
        teardown: function () {
            if (M.isDirty()) {
                M.confirmDiscard(function () { });
            }
            M.state = { files: [], current: null, savedContent: null, dirty: false, expandedFolders: {} };
            if (M.monacoInstance) {
                try { M.monacoInstance.dispose(); } catch (ignore) { }
                M.monacoInstance = null;
            }
            if (M.textarea) M.textarea.value = "";
        },
    });
})(window.VibeDialogs);
