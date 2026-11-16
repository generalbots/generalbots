"use strict";
/**
 * 06a_drive_actions.js - Drive file actions: upload, create folder,
 * context menu, rename, download, delete, preview, search.
 *
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
        container.innerHTML =
            '<div class="gb-drv-header">' +
                '<div class="gb-drv-header-left">' +
                    '<button class="gb-drv-back" id="gbDrvBack">' + I("caret-left", 14) +
                    ' <span>' + D.esc(t("drive.back", "Voltar")) + '</span></button>' +
                    '<span class="gb-drv-title">Drive</span>' +
                '</div>' +
            '</div>' +
            '<div class="gb-drv-breadcrumb" id="gbDrvBreadcrumb"></div>' +
            '<div class="gb-drv-toolbar" id="gbDrvToolbar">' +
                '<button class="gb-drv-btn gb-drv-btn-accent" id="gbDrvUploadBtn">' +
                    I("plus", 14) + ' <span>' + esc(t("drive.upload", "Enviar")) + '</span></button>' +
                '<input type="file" id="gbDrvFileInput" style="display:none" multiple />' +
                '<button class="gb-drv-btn" id="gbDrvNewFolder">' +
                    I("folder", 14) + ' <span>' + esc(t("drive.new-folder", "Nova pasta")) + '</span></button>' +
                '<div class="gb-drv-search-wrap">' +
                    '<svg class="gb-drv-search-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>' +
                    '<input type="text" class="gb-drv-search-input" id="gbDrvSearchInput" placeholder="' + esc(t("drive.search-placeholder", "Pesquisar...")) + '" />' +
                '</div>' +
            '</div>' +
            '<div class="gb-drv-col-header" id="gbDrvColHeader">' +
                '<button class="gb-drv-col-btn gb-drv-col-active" data-col="name">' + esc(t("drive.name", "Nome")) + ' <span class="gb-drv-sort-arrow"></span></button>' +
                '<button class="gb-drv-col-btn" data-col="size">' + esc(t("drive.size", "Tamanho")) + '</button>' +
                '<button class="gb-drv-col-btn" data-col="modified">' + esc(t("drive.modified", "Modificado")) + '</button>' +
                '<span></span>' +
            '</div>' +
            '<div class="gb-drv-list" id="gbDrvList"></div>' +
            '<div class="gb-drv-drop-overlay" id="gbDrvDropOverlay">' +
                '<span class="gb-drv-drop-text">' + esc(t("drive.drop-files", "Solte os arquivos aqui")) + '</span>' +
            '</div>';

        document.getElementById("gbDrvBack").addEventListener("click", function () { A.switchView("chat"); });

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
                if (e.key === "Escape") { searchInput.value = ""; if (D.searching) { D.searching = false; D.refreshFiles(); } }
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
        if (driveView) {
            driveView.addEventListener("dragenter", function (e) {
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
                if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length) {
                    D.uploadFiles(Array.prototype.slice.call(e.dataTransfer.files));
                }
            });
        }

        document.addEventListener("keydown", function (e) {
            if (A.currentView !== "drive") return;
            if (e.key === "Escape") { if (D.ctxEl) { D.closeCtx(); return; } A.switchView("chat"); }
        });
        document.addEventListener("click", function () { D.closeCtx(); });
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

    D.uploadFiles = function (files) {
        if (!files || !files.length) return;
        var toast = showUploadToast(files.length);
        var done = 0;
        var failed = 0;
        var failures = [];

        (function next(i) {
            if (i >= files.length) {
                if (failed === 0) {
                    updateUploadToast(toast, files.length, files.length,
                        A.t("drive.upload-done", "Envio concluido"));
                    setTimeout(function () { removeToast(); }, 2000);
                } else {
                    var label = failures.slice(0, 2).join("; ");
                    updateUploadToast(toast, done, files.length,
                        failed + " " + A.t("drive.upload-failed", "falha(s)") + ": " + label);
                    setTimeout(function () { removeToast(); }, 6000);
                }
                D.refreshFiles();
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
            cleanup();
            if (!val) return;
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

    /* ---- Context menu ---- */
    D.showCtxMenu = function (evt, path, name, isDir) {
        D.closeCtx();
        D.ctxEl = document.createElement("div");
        D.ctxEl.className = "gb-drv-ctx";

        var items = [
            { label: A.t("drive.rename", "Renomear"), icon: "file", action: "rename" },
            { label: A.t("drive.download", "Baixar"), icon: "download", action: "download" },
            { label: A.t("drive.delete", "Excluir"), icon: "trash", action: "delete", danger: true }
        ];

        var html = "";
        for (var i = 0; i < items.length; i++) {
            html += '<button class="gb-drv-ctx-item' +
                (items[i].danger ? " gb-drv-ctx-danger" : "") +
                '" data-action="' + items[i].action + '">' +
                I(items[i].icon, 14) + '<span>' + esc(items[i].label) + '</span></button>';
        }
        D.ctxEl.innerHTML = html;
        document.body.appendChild(D.ctxEl);

        var x = evt.clientX;
        var y = evt.clientY;
        var w = D.ctxEl.offsetWidth;
        var h = D.ctxEl.offsetHeight;
        if (x + w > window.innerWidth) x = window.innerWidth - w - 8;
        if (y + h > window.innerHeight) y = window.innerHeight - h - 8;
        D.ctxEl.style.left = x + "px";
        D.ctxEl.style.top = y + "px";

        D.ctxEl.addEventListener("click", function (e) {
            var btn = e.target.closest("[data-action]");
            if (!btn) return;
            var action = btn.getAttribute("data-action");
            D.closeCtx();
            if (action === "rename") startRename(path, name);
            else if (action === "download") downloadFile(path, name);
            else if (action === "delete") confirmDelete(path, name, isDir);
        });
    };

    /* ---- Inline rename ---- */
    function startRename(path, oldName) {
        var row = document.querySelector('.gb-drv-row[data-path="' + CSS.escape(path) + '"]');
        if (!row) return;
        var nameCell = row.querySelector(".gb-drv-name-text");
        if (!nameCell || nameCell.querySelector("input")) return;
        var original = nameCell.textContent;
        var input = document.createElement("input");
        input.type = "text";
        input.className = "gb-drv-rename-input";
        input.value = original;
        nameCell.textContent = "";
        nameCell.appendChild(input);
        input.focus();
        input.select();

        function commit() {
            var val = input.value.trim();
            cleanup();
            if (val && val !== original) doRename(path, val);
            else nameCell.textContent = original;
        }
        function cleanup() {
            input.removeEventListener("keydown", onKey);
            input.removeEventListener("blur", onBlur);
        }
        function onKey(e) {
            if (e.key === "Enter") { e.preventDefault(); commit(); }
            if (e.key === "Escape") { e.preventDefault(); cleanup(); nameCell.textContent = original; }
        }
        function onBlur() { commit(); }
        input.addEventListener("keydown", onKey);
        input.addEventListener("blur", onBlur);
    }

    function doRename(path, newName) {
        var parentPath = path.substring(0, path.lastIndexOf("/"));
        var newPath = parentPath ? parentPath + "/" + newName : newName;
        D.req("/move", {
            method: "POST",
            body: JSON.stringify({
                source_bucket: D.bucket || undefined,
                source_path: path,
                dest_bucket: D.bucket || undefined,
                dest_path: newPath,
                scope: D.scope
            })
        }).then(function () { D.refreshFiles(); })
          .catch(function (e) {
              D.showToast(A.t("drive.rename-error", "Erro ao renomear") + ": " + (e.message || e), "error");
              D.refreshFiles();
          });
    }

    /* ---- Download ---- */
    function downloadFile(path, name) {
        A.apiFetch(D.API + "/download-binary", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                bucket: D.bucket || undefined,
                path: path,
                scope: D.scope
            })
        }).then(function (r) {
            if (!r.ok) throw new Error("HTTP " + r.status);
            return r.blob();
        }).then(function (blob) {
            if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
                window.GBShared.saveBlob(blob, name);
            } else {
                var url = URL.createObjectURL(blob);
                var a = document.createElement("a");
                a.href = url; a.download = name;
                document.body.appendChild(a); a.click(); a.remove();
                URL.revokeObjectURL(url);
            }
        }).catch(function (e) {
            D.showToast(A.t("drive.download-error", "Erro ao baixar") + ": " + (e.message || e), "error");
        });
    }

    /* ---- Delete (inline confirmation) ---- */
    function confirmDelete(path, name) {
        var row = document.querySelector('.gb-drv-row[data-path="' + CSS.escape(path) + '"]');
        if (!row) return;
        if (row.querySelector(".gb-drv-confirm")) return;
        var savedHtml = row.innerHTML;

        var bar = document.createElement("div");
        bar.className = "gb-drv-confirm";
        bar.innerHTML =
            '<span class="gb-drv-confirm-text">' +
                esc(A.t("drive.delete-confirm", "Excluir") + ' "' + name + '"?') +
            '</span>' +
            '<button class="gb-drv-confirm-btn gb-drv-confirm-danger" data-act="yes">' +
                esc(A.t("assistant.delete", "Excluir")) + '</button>' +
            '<button class="gb-drv-confirm-btn" data-act="no">' +
                esc(A.t("assistant.cancel", "Cancelar")) + '</button>';

        row.innerHTML = "";
        row.appendChild(bar);

        function restore() { row.innerHTML = savedHtml; D.bindOneRow(row); }

        bar.addEventListener("click", function (e) {
            e.stopPropagation();
            var act = e.target.closest("[data-act]");
            if (!act) return;
            if (act.getAttribute("data-act") === "no") { restore(); return; }
            act.disabled = true;
            act.textContent = "...";
            D.req("/trash/move", {
                method: "POST",
                body: JSON.stringify({
                    bucket: D.bucket || undefined,
                    path: path,
                    scope: D.scope
                })
            }).then(function () { D.refreshFiles(); })
              .catch(function (e) {
                  bar.innerHTML =
                      '<span class="gb-drv-confirm-text" style="color:#dc2626">' +
                      esc(e.message || "Error") + '</span>' +
                      '<button class="gb-drv-confirm-btn" data-act="no">OK</button>';
                  bar.querySelector('[data-act="no"]').addEventListener("click", function (ev) {
                      ev.stopPropagation(); restore();
                  });
              });
        });

        var cancelBtn = bar.querySelector('[data-act="no"]');
        if (cancelBtn) cancelBtn.focus();
    }

    /* ---- Preview (reuses 13_attachpanel.js) ---- */
    D.previewFile = function (path, name) {
        A.apiFetch(D.API + "/download-inline", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                bucket: D.bucket || undefined,
                path: path,
                scope: D.scope
            })
        }).then(function (r) {
            if (!r.ok) throw new Error("HTTP " + r.status);
            return r.blob();
        }).then(function (blob) {
            var file;
            try { file = new File([blob], name, { type: blob.type }); }
            catch (e) { file = blob; file.name = name; }
            if (typeof A.closeAttachPanel === "function") A.closeAttachPanel();
            var ghost = document.createElement("div");
            ghost.style.display = "none";
            document.body.appendChild(ghost);
            A.makeAttachClickable(ghost, file, {});
            ghost.click();
            ghost.remove();
        }).catch(function (e) {
            D.showToast(A.t("drive.preview-error", "Erro ao visualizar") + ": " + (e.message || e), "error");
        });
    };

    /* ---- Search ---- */
    D.searchFiles = function (query) {
        D.searching = true;
        D.showLoading();
        var params = new URLSearchParams();
        params.set("query", query);
        params.set("scope", D.scope);
        params.set("all_buckets", "false");
        if (D.bucket) params.set("bucket", D.bucket);

        D.req("/search?" + params.toString())
            .then(function (files) {
                D.files = D.sortFiles(files || []);
                D.renderFileList();
            })
            .catch(function (e) {
                console.error("[drive] search failed:", e);
                D.showError(e.message);
            });
    };
})();
