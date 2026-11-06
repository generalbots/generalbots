/* Vibe Code dialog — split from vibe/vibe-dialog-code.js for #1731 */
(function (D) {
    "use strict";
    var M = D.__code = D.__code || {};
    var D = window.VibeDialogs;
    M.state = { files: [], current: null, savedContent: null, dirty: false, expandedFolders: {} };
    M.monacoInstance = null;
    M.monacoLang = "plaintext";
    M.textarea = null;
    M.editorHost = null;

    /* ── Inline SVG icons (stroke style, matches the Vibe shell toolbar) ── */
    M.ICONS = {
        save: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>',
        newFile: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="12" x2="12" y2="18"/><line x1="9" y1="15" x2="15" y2="15"/></svg>',
        folder: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>',
        file: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="9" y1="13" x2="15" y2="13"/><line x1="9" y1="17" x2="13" y2="17"/></svg>',
    };

    function svgBtnIcon(key) {
        var svg = M.ICONS[key] || "";
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
        box.appendChild(M.editorArea());
        box.appendChild(M.statusBar());
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
        save.innerHTML = M.svgBtnIcon("save") + '<span>Save (Ctrl+S)</span>';
        save.addEventListener("click", M.saveFile);
        var newBtn = D.el("button", "vibe-btn");
        newBtn.innerHTML = M.svgBtnIcon("newFile") + '<span>New File</span>';
        newBtn.addEventListener("click", M.newFile);
        toolbar.appendChild(name);
        toolbar.appendChild(lang);
        toolbar.appendChild(spacer);
        toolbar.appendChild(newBtn);
        toolbar.appendChild(save);
        wrap.appendChild(toolbar);

        // Monaco host — replaced by the textarea fallback if Monaco fails.
        M.editorHost = D.el("div", "vibe-code-monaco");
        M.editorHost.id = "vibeMonacoHost";
        M.editorHost.style.cssText = "flex:1;min-height:0;display:none;";
        wrap.appendChild(M.editorHost);

        M.textarea = D.el("textarea", "vibe-textarea");
        M.textarea.id = "vibeCodeContent";
        M.textarea.placeholder = "Open a file from the workspace to edit it.\n\nFiles live in the project workspace (VIBE_WORKSPACE_ROOT).";
        M.textarea.spellcheck = false;
        M.textarea.style.display = "none";
        wrap.appendChild(M.textarea);
        return wrap;
    }

    function statusBar() {
        var bar = D.el("div", "vibe-browser-status");
        bar.id = "vibeCodeStatus";
        bar.innerHTML = '<span id="vibeCodeStatusMsg">editor ready</span>';
        return bar;
    }

    M.svgBtnIcon = svgBtnIcon;
    M.selectedProjectId = selectedProjectId;
    M.sidebar = sidebar;
    M.main = main;
    M.editorArea = editorArea;
    M.statusBar = statusBar;
})(window.VibeDialogs);
