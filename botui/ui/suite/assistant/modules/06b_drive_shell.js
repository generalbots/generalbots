"use strict";
/**
 * 06b_drive_shell.js - Drive shell rendering, upload, folder creation.
 * Split from 06a_drive_actions.js to stay under 450 lines each.
 * Reads shared state from window.GBDrive (set by 06_drive.js).
 */
(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };
    var D = window.GBDrive;
    if (!D) { console.error("[drive] 06_drive.js not loaded"); return; }
    var esc = D.esc;

    /* ================================================================
       Shell: header, breadcrumb, toolbar, column headers, list area
       ================================================================ */
    D.renderShell = function (container) {
        var t = A.t.bind(A);
        var portuguese = document.documentElement.lang.toLowerCase().startsWith("pt");
        var tabs =
            '<button type="button" class="gb-drv-tab active" data-tab="files" aria-pressed="true">' +
                I("folder", 15) + '<span>' + esc(t("drive.my-files", "My files")) + '</span></button>' +
            '<button type="button" class="gb-drv-tab" data-tab="inbox" aria-pressed="false">' +
                I("paperclip", 15) + '<span>' + esc(t("drive.chat-attachments", "Conversation attachments")) + '</span></button>';
        container.innerHTML =
            '<div class="gb-drv-header">' +
                '<div><span class="gb-label">Drive</span>' +
                '<h1 class="gb-drv-title">' + (portuguese ? "Seus arquivos, neste computador." : "Your files, on this computer.") + '</h1></div>' +
                '<div class="gb-drv-tabs gb-seg" id="gbDrvTabs">' + tabs + '</div>' +
            '</div>' +
            '<div class="gb-drv-panel">' +
            '<div class="gb-drv-toolbar" id="gbDrvToolbar">' +
                '<div class="gb-drv-location">' + I("folder", 15) +
                    '<div class="gb-drv-breadcrumb" id="gbDrvBreadcrumb"></div></div>' +
                '<div class="gb-drv-search-wrap">' +
                    '<span class="gb-drv-search-icon" aria-hidden="true">' + I("magnifying-glass", 14) + '</span>' +
                    '<input type="search" class="gb-drv-search-input" id="gbDrvSearchInput" aria-label="' + esc(t("drive.search-placeholder", "Search")) +
                    '" placeholder="' + esc(t("drive.search-placeholder", "Search...")) + '" />' +
                '</div>' +
                '<button type="button" class="gb-btn" id="gbDrvNewFolder">' +
                    I("folder", 14) + ' <span>' + esc(t("drive.new-folder", "New folder")) + '</span></button>' +
                '<button type="button" class="gb-btn gb-btn-pri" id="gbDrvUploadBtn">' +
                    I("plus", 14) + ' <span>' + esc(t("drive.upload", "Upload")) + '</span></button>' +
                '<input type="file" id="gbDrvFileInput" hidden multiple />' +
            '</div>' +
            '<div class="gb-drv-col-header" id="gbDrvColHeader">' +
                '<button class="gb-drv-col-btn gb-drv-col-active" data-col="name">' + esc(t("drive.name", "Nome")) + ' <span class="gb-drv-sort-arrow"></span></button>' +
                '<button class="gb-drv-col-btn" data-col="size">' + esc(t("drive.size", "Tamanho")) + '</button>' +
                '<button class="gb-drv-col-btn" data-col="modified">' + esc(t("drive.modified", "Modificado")) + '</button>' +
                '<span></span>' +
            '</div>' +
            '<div class="gb-drv-list" id="gbDrvList"></div>' +
            '</div>' +
            '<p class="gb-drv-privacy">' + I("shield-check", 15) +
                (portuguese ? "Os arquivos ficam neste computador. Eles só vão a um modelo quando você usa um deles numa conversa." :
                    "Files stay on this computer. They go to a model only when you use them in a conversation.") + '</p>' +
            '<div class="gb-drv-drop-overlay" id="gbDrvDropOverlay">' +
                '<span class="gb-drv-drop-text">' + esc(t("drive.drop-files", "Solte os arquivos aqui")) + '</span>' +
            '</div>';

        var rail = document.getElementById("gbDriveNav");
        if (rail) {
            rail.innerHTML = '<span class="gb-strip">Drive</span>' + tabs +
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
                if (!q) { if (D.searching) { D.searching = false; D.refreshFiles(); } return; }
                debounce = setTimeout(function () { D.searchFiles(q); }, 400);
            });
            searchInput.addEventListener("keydown", function (e) {
                if (e.key === "Escape") {
                    e.stopPropagation();
                    clearTimeout(debounce);
                    searchInput.value = "";
                    if (D.searching) { D.searching = false; D.refreshFiles(); }
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

    var MAX_DRIVE_UPLOAD = 20 * 1024 * 1024; /* 20 MB */

    D.uploadFiles = function (files) {
        if (!files || !files.length) return;
        /* Reject files over the limit before starting the upload. */
        var tooBig = [];
        var ok = [];
        for (var fi = 0; fi < files.length; fi++) {
            if (files[fi].size > MAX_DRIVE_UPLOAD) {
                tooBig.push(files[fi].name + " (" + D.fmtBytes(files[fi].size) + ")");
            } else {
                ok.push(files[fi]);
            }
        }
        if (tooBig.length) {
            D.showToast(
                A.t("drive.file-too-big", "Arquivo grande demais (limite de 20 MB): ") + tooBig.join(", "),
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
                    if (!D.inboxMode) D.refreshFiles();
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
