"use strict";
/**
 * 06a_drive_actions.js - Drive file actions: context menu, rename,
 * download, delete, preview, search.
 * Shell rendering, upload and folder creation in 06b_drive_shell.js.
 *
 * Reads shared state from window.GBDrive (set by 06_drive.js).
 */
(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };
    var D = window.GBDrive;
    if (!D) { console.error("[drive] 06_drive.js not loaded"); return; }
    var esc = D.esc;

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
