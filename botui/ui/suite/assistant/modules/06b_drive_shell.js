"use strict";
/**
 * 06b_drive_shell.js - Drive shell rendering, upload, folder creation.
 * Split from 06a_drive_actions.js to stay under 450 lines each.
 * Reads shared state from window.GBDrive (set by 06_drive.js).
 */
(function () {
    var A = window.GBAssistant;
    var D = window.GBDrive;
    if (!D) { console.error("[drive] 06_drive.js not loaded"); return; }
    var esc = D.esc;

    /* ================================================================
       Shell: header, breadcrumb, toolbar, column headers, list area
       ================================================================ */
    var P = D.PATHS = {
        folder: "M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z",
        clip: "M21.44 11.05 12.25 20.24a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48",
        upload: "M12 15V3M7 8l5-5 5 5M5 21h14",
        shield: "M12 2 4 6v6c0 5 3.4 8.6 8 10 4.6-1.4 8-5 8-10V6z",
        sort: "m6 15 6-6 6 6"
    };
    /* Inline 24-unit stroke icon; `inner` is a path string or ready SVG markup. */
    D.svg = function (inner, size, stroke, cap) {
        var body = inner.charAt(0) === "<" ? inner : '<path d="' + inner + '"/>';
        return '<svg width="' + size + '" height="' + size + '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="' +
            stroke + '" stroke-linecap="' + (cap || "round") + '" stroke-linejoin="round" aria-hidden="true">' + body + '</svg>';
    };
    D.railIcon = function (path) { return '<span class="gb-drv-rail-icon">' + D.svg(path, 13, 1.7) + '</span>'; };

    D.renderShell = function (container) {
        var t = A.t.bind(A);
        var portuguese = document.documentElement.lang.toLowerCase().startsWith("pt");
        function tab(id, label, icon) {
            return '<button type="button" class="gb-drv-tab' + (id === "files" ? " active" : "") + '" data-tab="' + id +
                '" aria-pressed="' + (id === "files") + '">' + (icon ? D.railIcon(icon) : "") +
                '<span class="gb-drv-tab-label">' + esc(label) + '</span>' + (icon ? "" : '<span class="gb-drv-count"></span>') + '</button>';
        }
        var myFiles = t("drive.my-files", "My files"), attachments = t("drive.chat-attachments", "Conversation attachments");
        var colLabel = function (key, fallback) { return esc(t(key, fallback).toUpperCase()); };
        var sortArrow = '<span class="gb-drv-sort-arrow">' + D.svg(P.sort, 10, 3, "square") + '</span>';
        container.innerHTML =
            '<div class="gb-drv-header">' +
                '<div class="gb-drv-heading"><span class="gb-drv-eyebrow"><span class="gb-drv-slash">//</span>DRIVE</span>' +
                '<h1 class="gb-drv-title">' + (portuguese ? "Seus arquivos, neste computador." : "Your files, on this computer.") + '</h1></div>' +
                '<div class="gb-drv-tabs" id="gbDrvTabs">' + tab("files", myFiles) + tab("inbox", attachments) + '</div>' +
            '</div>' +
            '<div class="gb-drv-panel">' +
            '<div class="gb-drv-toolbar" id="gbDrvToolbar">' +
                '<div class="gb-drv-location">' + D.svg(P.folder, 15, 1.7) +
                    '<div class="gb-drv-breadcrumb" id="gbDrvBreadcrumb"></div></div>' +
                '<div class="gb-drv-search-wrap">' +
                    D.svg('<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>', 14, 1.7) +
                    '<input type="search" class="gb-drv-search-input" id="gbDrvSearchInput" aria-label="' + esc(t("drive.search-placeholder", "Search")) +
                    '" placeholder="' + (portuguese ? "Pesquisar…" : "Search…") + '" />' +
                '</div>' +
                '<button type="button" class="gb-drv-tbtn" id="gbDrvNewFolder">' +
                    D.svg(P.folder + "M12 10v6M9 13h6", 13, 2) + '<span>' + esc(t("drive.new-folder", "New folder")) + '</span></button>' +
                '<button type="button" class="gb-drv-tbtn gb-drv-tbtn-pri" id="gbDrvUploadBtn">' +
                    D.svg(P.upload, 13, 2.2) + '<span>' + esc(t("drive.upload", "Upload")) + '</span></button>' +
                '<input type="file" id="gbDrvFileInput" hidden multiple />' +
            '</div>' +
            '<div class="gb-drv-col-header" id="gbDrvColHeader">' +
                '<button type="button" class="gb-drv-col-btn gb-drv-col-active" data-col="name">' + colLabel("drive.name", "Name") + sortArrow + '</button>' +
                '<button type="button" class="gb-drv-col-btn" data-col="size">' + colLabel("drive.size", "Size") + sortArrow + '</button>' +
                '<button type="button" class="gb-drv-col-btn" data-col="modified">' + colLabel("drive.modified", "Modified") + sortArrow + '</button>' +
                '<span></span>' +
            '</div>' +
            '<div class="gb-drv-list" id="gbDrvList"></div>' +
            '</div>' +
            '<p class="gb-drv-privacy">' + D.svg(P.shield, 14, 1.8) + '<span>' +
                (portuguese ? "Os arquivos ficam neste computador. Eles só vão a um modelo quando você usa um deles numa conversa." :
                    "Files stay on this computer. They go to a model only when you use them in a conversation.") + '</span></p>' +
            '<div class="gb-drv-drop-overlay" id="gbDrvDropOverlay">' +
                '<span class="gb-drv-drop-text">' + (portuguese ? "Solte para enviar ao Drive" : "Drop to upload to Drive") + '</span>' +
            '</div>';

        var rail = document.getElementById("gbDriveNav");
        if (rail) {
            rail.innerHTML = '<span class="gb-strip">DRIVE</span>' + tab("files", myFiles, P.folder) + tab("inbox", attachments, P.clip) +
                '<div class="gb-drv-rail-folders" id="gbDrvRailFolders" hidden></div>';
            rail.onclick = switchTab;
        }
        var tabsEl = document.getElementById("gbDrvTabs");
        tabsEl.addEventListener("click", switchTab);
        function switchTab(event) {
            var tab = event.target.closest("[data-tab]");
            if (!tab || tab.getAttribute("aria-pressed") === "true") return;
            var inbox = tab.dataset.tab === "inbox";
            clearTimeout(debounce);
            searchInput.value = "";
            D.searching = false;
            D.selected = null;
            D.path = "";
            container.classList.toggle("gb-drv-inbox", inbox);
            [tabsEl, rail].filter(Boolean).forEach(function (group) {
                group.querySelectorAll("[data-tab]").forEach(function (button) {
                    var active = button.dataset.tab === tab.dataset.tab;
                    button.classList.toggle("active", active);
                    button.setAttribute("aria-pressed", String(active));
                });
            });
            if (inbox) {
                D.loadInboxFiles();
            } else {
                D.inboxMode = false;
                D.scope = "user";
                D.refreshFiles();
            }
        }

        var uploadBtn = document.getElementById("gbDrvUploadBtn");
        var fileInput = document.getElementById("gbDrvFileInput");
        if (uploadBtn && fileInput) {
            uploadBtn.addEventListener("click", function () { fileInput.click(); });
            fileInput.addEventListener("change", function () {
                if (fileInput.files && fileInput.files.length) {
                    D.uploadFiles(Array.prototype.slice.call(fileInput.files));
                    fileInput.value = "";
                }
            });
        }

        var newFolderBtn = document.getElementById("gbDrvNewFolder");
        if (newFolderBtn) newFolderBtn.addEventListener("click", function () { D.createFolderPrompt(); });

        var searchInput = document.getElementById("gbDrvSearchInput");
        if (searchInput) {
            var debounce = null;
            searchInput.addEventListener("input", function () {
                clearTimeout(debounce);
                var q = searchInput.value.trim();
                if (!q) { if (D.searching) { D.searching = false; D.reloadList(); } return; }
                debounce = setTimeout(function () { D.searchFiles(q); }, 400);
            });
            searchInput.addEventListener("keydown", function (e) {
                if (e.key === "Escape") {
                    e.stopPropagation();
                    clearTimeout(debounce);
                    searchInput.value = "";
                    if (D.searching) { D.searching = false; D.reloadList(); }
                }
            });
        }

        var colHeader = document.getElementById("gbDrvColHeader");
        if (colHeader) {
            colHeader.addEventListener("click", function (e) {
                var btn = e.target.closest("[data-col]");
                if (!btn) return;
                var col = btn.getAttribute("data-col");
                if (D.sort.col === col) D.sort.asc = !D.sort.asc;
                else { D.sort.col = col; D.sort.asc = true; }
                D.files = D.sortFiles(D.files);
                D.renderFileList();
            });
        }

        var driveView = document.getElementById("gbDriveView");
        if (driveView && !D.shellListenersBound) {
            driveView.addEventListener("dragenter", function (e) {
                if (D.inboxMode) return;
                e.preventDefault(); D.dragCount++;
                var ov = document.getElementById("gbDrvDropOverlay");
                if (ov) ov.classList.add("active");
            });
            driveView.addEventListener("dragleave", function (e) {
                e.preventDefault(); D.dragCount--;
                if (D.dragCount <= 0) { D.dragCount = 0; var ov = document.getElementById("gbDrvDropOverlay"); if (ov) ov.classList.remove("active"); }
            });
            driveView.addEventListener("dragover", function (e) { e.preventDefault(); });
            driveView.addEventListener("drop", function (e) {
                e.preventDefault(); D.dragCount = 0;
                var ov = document.getElementById("gbDrvDropOverlay");
                if (ov) ov.classList.remove("active");
                if (D.inboxMode) return;
                if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length) {
                    D.uploadFiles(Array.prototype.slice.call(e.dataTransfer.files));
                }
            });
        }

        if (!D.shellListenersBound) {
            document.addEventListener("keydown", function (e) {
                if (A.currentView !== "drive" || e.defaultPrevented || e.key !== "Escape") return;
                if (D.ctxEl) { D.closeCtx(true); return; }
                A.switchView("chat");
            });
            document.addEventListener("click", function () { D.closeCtx(); });
            D.shellListenersBound = true;
        }
    };

    /* ================================================================
       Upload
       ================================================================ */
    function readFileAsBase64(file) {
        return new Promise(function (resolve, reject) {
            var reader = new FileReader();
            reader.onload = function () {
                var result = reader.result || "";
                var idx = result.indexOf(",");
                resolve(idx >= 0 ? result.substring(idx + 1) : result);
            };
            reader.onerror = function () { reject(new Error("Read failed")); };
            reader.readAsDataURL(file);
        });
    }

    /* /api/files/write takes the file as base64 inside a JSON body of at
       most 100 MiB (server upload policy, Drive class): 75 MB of file. */
    var DRIVE_BODY_MAX = 100 * 1024 * 1024;
    D.MAX_UPLOAD_MB = 75;

    function overDriveLimit(file) {
        var path = (D.path ? D.path + "/" : "") + file.name;
        return Math.ceil(file.size / 3) * 4 + path.length * 4 + 1024 > DRIVE_BODY_MAX;
    }

    D.reloadList = function () {
        if (D.inboxMode) D.loadInboxFiles();
        else D.refreshFiles();
    };

    D.uploadFiles = function (files) {
        if (!files || !files.length) return;
        /* Reject files over the limit before starting the upload. */
        var tooBig = [];
        var ok = [];
        for (var fi = 0; fi < files.length; fi++) {
            if (overDriveLimit(files[fi])) {
                tooBig.push(files[fi].name + " (" + D.fmtBytes(files[fi].size) + ")");
            } else {
                ok.push(files[fi]);
            }
        }
        if (tooBig.length) {
            D.showToast(
                A.t("drive.file-too-big", "Arquivo grande demais para o Drive (limite de 75 MB por arquivo): ") + tooBig.join(", "),
                "error"
            );
        }
        files = ok;
        if (!files.length) return;
        /* Track which names already exist so the toast can say "replaced". */
        var existingNames = {};
        for (var ei = 0; ei < D.files.length; ei++) {
            existingNames[(D.files[ei].name || "").toLowerCase()] = true;
        }
        var replaced = 0;
        var toast = showUploadToast(files.length);
        var done = 0;
        var failed = 0;
        var failures = [];

        (function next(i) {
            if (i >= files.length) {
                if (failed === 0) {
                    var doneMsg = A.t("drive.upload-done", "Envio concluido");
                    if (replaced > 0) {
                        doneMsg += " (" + replaced + " " + A.t("drive.replaced", "substituido(s)") + ")";
                    }
                    updateUploadToast(toast, files.length, files.length, doneMsg);
                    setTimeout(function () { removeToast(); }, replaced > 0 ? 4000 : 2000);
                } else {
                    var label = failures.slice(0, 2).join("; ");
                    updateUploadToast(toast, done, files.length,
                        failed + " " + A.t("drive.upload-failed", "falha(s)") + ": " + label);
                    setTimeout(function () { removeToast(); }, 6000);
                }
                if (D.inboxMode) {
                    var filesTab = document.querySelector('#gbDrvTabs [data-tab="files"]');
                    if (filesTab) filesTab.click();
                } else D.refreshFiles();
                return;
            }
            updateUploadToast(toast, i, files.length, files[i].name);
            readFileAsBase64(files[i]).then(function (b64) {
                var filePath = D.path ? D.path + "/" + files[i].name : files[i].name;
                return D.req("/write", {
                    method: "POST",
                    body: JSON.stringify({
                        bucket: D.bucket || undefined,
                        path: filePath,
                        content: b64,
                        scope: D.scope
                    })
                });
            }).then(function () {
                done++;
                if (existingNames[files[i].name.toLowerCase()]) { replaced++; }
                next(i + 1);
            }).catch(function (e) {
                failed++;
                failures.push(files[i].name + " (" + (e.message || "error") + ")");
                next(i + 1);
            });
        })(0);
    };

    function showUploadToast(total) {
        removeToast();
        var el = document.createElement("div");
        el.className = "gb-drv-upload-toast"; el.id = "gbDrvUploadToast";
        el.innerHTML = '<div class="gb-drv-upload-label" id="gbDrvUploadLabel">' + esc(A.t("drive.uploading", "Enviando...")) + '</div><div class="gb-drv-upload-bar-bg"><div class="gb-drv-upload-bar" id="gbDrvUploadBar"></div></div><div class="gb-drv-upload-detail" id="gbDrvUploadDetail">0 / ' + total + '</div>';
        document.body.appendChild(el); return el;
    }
    function updateUploadToast(toast, current, total, text) {
        if (!toast) return;
        var pct = Math.round((current / total) * 100);
        var bar = document.getElementById("gbDrvUploadBar");
        var label = document.getElementById("gbDrvUploadLabel");
        var detail = document.getElementById("gbDrvUploadDetail");
        if (bar) bar.style.width = pct + "%";
        if (label) label.textContent = text ? (text.length > 35 ? text.substring(0, 32) + "..." : text) : "";
        if (detail) detail.textContent = current + " / " + total;
    }
    function removeToast() { var el = document.getElementById("gbDrvUploadToast"); if (el) el.remove(); }

    /* ---- Create folder (inline field) ---- */
    D.createFolderPrompt = function () {
        var toolbar = document.getElementById("gbDrvToolbar");
        if (!toolbar || toolbar.querySelector(".gb-drv-inline-input")) return;
        var wrap = document.createElement("div");
        wrap.className = "gb-drv-inline-input";
        var input = document.createElement("input");
        input.type = "text";
        input.className = "gb-drv-rename-input";
        input.placeholder = A.t("drive.folder-prompt", "Nome da nova pasta");
        input.setAttribute("aria-label", A.t("drive.folder-prompt", "Nome da nova pasta"));
        wrap.appendChild(input);
        var ok = document.createElement("button");
        ok.className = "gb-drv-btn gb-drv-btn-accent gb-drv-btn-sm";
        ok.textContent = "OK";
        wrap.appendChild(ok);
        var cancel = document.createElement("button");
        cancel.className = "gb-drv-btn gb-drv-btn-sm";
        cancel.textContent = A.t("assistant.cancel", "Cancelar");
        wrap.appendChild(cancel);
        toolbar.appendChild(wrap);
        input.focus();

        function cleanup() { wrap.remove(); }
        function commit() {
            var val = input.value.trim();
            if (!val) {
                input.classList.add("gb-drv-input-error");
                input.focus();
                return;
            }
            /* Check for duplicate name */
            var exists = D.files.some(function (f) {
                return (f.name || "").toLowerCase() === val.toLowerCase();
            });
            if (exists) {
                D.showToast(A.t("drive.folder-exists", "Já existe um item com esse nome"), "error");
                input.classList.add("gb-drv-input-error");
                input.select();
                return;
            }
            cleanup();
            D.req("/createFolder", { method: "POST", body: JSON.stringify({ bucket: D.bucket || undefined, path: D.path || "", name: val, scope: D.scope }) })
                .then(function () { D.refreshFiles(); })
                .catch(function (e) { D.showToast(A.t("drive.folder-error", "Erro ao criar pasta") + ": " + (e.message || e), "error"); });
        }
        ok.addEventListener("click", commit);
        cancel.addEventListener("click", cleanup);
        input.addEventListener("keydown", function (e) {
            if (e.key === "Enter") { e.preventDefault(); commit(); }
            if (e.key === "Escape") { e.preventDefault(); cleanup(); }
        });
    };
})();
