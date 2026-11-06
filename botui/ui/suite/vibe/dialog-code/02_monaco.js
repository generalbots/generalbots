/* Vibe Code dialog — split from vibe/vibe-dialog-code.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__code;
    var D = window.VibeDialogs;
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
                else if (tries > 100) { clearInterval(t); M.useTextareaFallback(); }
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
        script.onerror = function () { M.useTextareaFallback(); };
        document.head.appendChild(script);
    }

    function useTextareaFallback() {
        if (!M.editorHost || !M.textarea) return;
        M.editorHost.style.display = "none";
        M.textarea.style.display = "block";
    }

    function setValue(content) {
        if (M.monacoInstance) {
            M.monacoInstance.setValue(content == null ? "" : String(content));
        } else {
            M.textarea.value = content == null ? "" : String(content);
        }
        // #1395 — reset the dirty baseline whenever a file (re)loads.
        M.state.savedContent = M.getValue();
        M.setDirty(false);
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
        return M.state.current != null && M.getValue() !== M.state.savedContent;
    }

    function setDirty(dirty) {
        var chip = document.getElementById("vibeCodeFileName");
        if (chip) {
            var base = M.displayName(M.state.current || "");
            chip.textContent = dirty ? base + " \u25CF" : base;
        }
        M.state.dirty = !!dirty;
    }

    function getValue() {
        if (M.monacoInstance) return M.monacoInstance.getValue();
        return M.textarea.value;
    }

    function showMonacoFor(name) {
        M.monacoLang = M.langFor(name);
        var langEl = document.getElementById("vibeCodeFileLang");
        if (langEl) {
            langEl.textContent = M.monacoLang.toUpperCase();
            langEl.className = "vibe-status warn";
        }
        if (!window.monaco || !M.editorHost) { M.useTextareaFallback(); return; }
        M.editorHost.style.display = "block";
        M.textarea.style.display = "none";
        if (M.monacoInstance) {
            M.monacoInstance.dispose();
            M.monacoInstance = null;
        }
        M.monacoInstance = window.monaco.editor.create(M.editorHost, {
            value: "",
            language: M.monacoLang,
            theme: "vs-dark",
            fontSize: 12,
            fontFamily: '"Fira Code", Consolas, monospace',
            automaticLayout: true,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            tabSize: 4,
        });
        if (M.textarea.value) M.monacoInstance.setValue(M.textarea.value);
        // #1395 — mark the chip dirty as soon as the user edits.
        var markDirty = function () { M.setDirty(true); };
        if (M.monacoInstance) {
            M.monacoInstance.onDidChangeModelContent(markDirty);
        } else {
            M.textarea.removeEventListener("input", markDirty);
            M.textarea.addEventListener("input", markDirty);
        }
    }

    M.langFor = langFor;
    M.loadMonaco = loadMonaco;
    M.useTextareaFallback = useTextareaFallback;
    M.setValue = setValue;
    M.setStatus = setStatus;
    M.isDirty = isDirty;
    M.setDirty = setDirty;
    M.getValue = getValue;
    M.showMonacoFor = showMonacoFor;
})(window.VibeDialogs);
